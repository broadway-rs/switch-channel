use criterion::*;
use criterion::async_executor::AsyncStdExecutor;
use switch_channel::{Switcher, err, async_channel::async_std::*};
use futures::{future, FutureExt};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use async_std::task;

async fn add_switch_loop(value: &mut usize, add: Result<usize, err::recv::RecvError>, add_receiver: &SwitchReceiver<usize, 2, true>){
    if let Ok(add) = add{
        *value += add;
        task::sleep(std::time::Duration::from_millis(10)).await;
    }
    while let Ok(add) = add_receiver.switch_xor(1).recv().await{
        *value += add;
        task::sleep(std::time::Duration::from_millis(10)).await;
    }
}

async fn parallel_switch_loop(value: &AtomicUsize, para: Result<(), err::recv::RecvError>, para_receiver: &SwitchReceiver<(), 2, true>){
    if let Ok(_) = para{
        value.fetch_add(1, Ordering::SeqCst);
    }
    // Run these in parallel
    future::join_all(para_receiver
        .switch_xor(1)
        .into_iter()
        .map(|_| task::sleep(std::time::Duration::from_millis(10)))).await;
}

async fn parallel_use_case(add_target: usize, par_target: usize) -> Result<(), Box<dyn std::error::Error>>{
    let (add_sender, add_receiver) = unbounded::<_, 2, false, true>();
    let (para_sender, para_receiver) = unbounded::<_, 2, false, true>();

    let handle = task::spawn(async move {
        for i in 0usize..add_target{
            let _ = add_sender.send(1).await;
        };
        for i in 0usize..par_target{
            let _ = para_sender.send(()).await;
        }
        add_sender.close();
        para_sender.close();
    });

    let mut adder_value: usize = 0;
    let para_adder_value = AtomicUsize::new(0);

    let mut add_recv_fut = None;
    let mut para_recv_fut = None;

    loop {
        if let None = add_recv_fut{
            if !add_receiver.is_empty() || !add_receiver.is_closed(){
                add_recv_fut = Some(Box::pin(add_receiver.recv()));
            }
        }
        if let None = para_recv_fut {
            if !para_receiver.is_empty() || !para_receiver.is_closed(){
                para_recv_fut = Some(Box::pin(para_receiver.recv()));
            }
        }

        let select_fut = match (add_recv_fut.take(), para_recv_fut.take()){
            (Some(add_fut), Some(para_fut)) =>{
                match future::select(add_fut, para_fut).await{
                    future::Either::Left((add, para_fut)) => {
                        add_switch_loop(&mut adder_value, add, &add_receiver).await;
                        para_recv_fut = Some(para_fut);
                    },
                    future::Either::Right((para, add_fut)) => {
                        parallel_switch_loop(&para_adder_value, para, &para_receiver).await;
                        add_recv_fut = Some(add_fut);
                    },
                }
            },
            (Some(add_fut), None) => add_switch_loop(&mut adder_value, add_fut.await, &add_receiver).await,
            (None, Some(para_fut)) => parallel_switch_loop(&para_adder_value, para_fut.await, &para_receiver).await,
            (None, None) => break,
        };
    };

    handle.await;
    assert_eq!(adder_value, add_target);
    //assert_eq!(para_adder_value.load(Ordering::SeqCst), par_target);
    Ok(())
}

enum Message{
    Add,
    ParAdd,
}

/// This is essentially Orleans world
async fn seq_use_case(add_target: usize, par_target: usize) -> Result<(), Box<dyn std::error::Error>>{
    let (add_sender, add_receiver) = unbounded::<Message, 1, false, true>();
    
    let mut adder_value: usize = 0;
    let para_adder_value = AtomicUsize::new(0);

    let handle = task::spawn(async move {
        for _ in 0usize..add_target{
            let _ = add_sender.send(Message::Add).await;
        };
        for _ in 0usize..par_target{
            let _ = add_sender.send(Message::ParAdd).await;
        }
        add_sender.close();
    });

    while !add_receiver.is_closed() || !add_receiver.is_empty(){
        for m in add_receiver
            .get_guard()
            .into_iter(){
                match m{
                    Message::Add => {
                        adder_value += 1;
                        task::sleep(std::time::Duration::from_millis(10)).await;
                },
                    Message::ParAdd => {task::sleep(std::time::Duration::from_millis(10)).await},
                }
            }
    }

    handle.await;
    assert_eq!(adder_value, add_target);
    //assert_eq!(para_adder_value.load(Ordering::SeqCst), par_target);
    Ok(())
}

fn par_bench(c: &mut Criterion){
    let mut para = c.benchmark_group("Para");
    para
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(30));
    para.bench_function("para_10_90", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| parallel_use_case(10, 90))
    });
    para.bench_function("para_40_60", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| parallel_use_case(40, 60))
    });
    para.bench_function("para_60_40", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| parallel_use_case(60, 40))
    });
    para.bench_function("para_90_10", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| parallel_use_case(90, 10))
    });
}

fn seq_bench(c: &mut Criterion){
    let mut seq = c.benchmark_group("Seq");
    seq
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(30));
    seq.bench_function("seq_10_90", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| seq_use_case(10, 90))
    });
    seq.bench_function("seq_40_60", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| seq_use_case(40, 60))
    });
    seq.bench_function("seq_60_40", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| seq_use_case(60, 40))
    });
    seq.bench_function("seq_90_10", move |b|{
        b.to_async(AsyncStdExecutor).iter(|| seq_use_case(90, 10))
    });
}

criterion_group!(benches, par_bench);
criterion_main!(benches);