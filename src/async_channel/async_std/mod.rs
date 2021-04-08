mod switch_receiver;
mod switch_sender;

use std::convert::TryInto;
use core::sync::atomic::AtomicUsize;
use std::sync::Arc;
use core::iter::repeat_with;
pub use switch_receiver::{SwitchReceiver};
pub use switch_sender::{SwitchSender};


pub fn bounded<T, const N: usize, const S: bool, const P: bool>(cap: usize) -> (SwitchSender<T, N, S>, SwitchReceiver<T, N, P>){
    use async_std::channel::{bounded, Sender, Receiver};
    
    let (senders, receivers): (Vec<Sender<T>>, Vec<Receiver<T>>) = repeat_with(|| bounded(cap)).take(N).unzip();

    let switch = Arc::new(AtomicUsize::new(0));
    (
        SwitchSender{
            count: switch.clone(),
            senders: senders.try_into().unwrap()
        },
        SwitchReceiver{
            count: switch.clone(),
            receivers: receivers.try_into().unwrap()
        }
    )
}

pub fn unbounded<T, const N: usize, const S: bool, const P: bool>() -> (SwitchSender<T, N, S>, SwitchReceiver<T, N, P>){
    use async_std::channel::{unbounded, Sender, Receiver};
    
    let (senders, receivers): (Vec<Sender<T>>, Vec<Receiver<T>>) = repeat_with(|| unbounded()).take(N).unzip();

    let switch = Arc::new(AtomicUsize::new(0));
    (
        SwitchSender{
            count: switch.clone(),
            senders: senders.try_into().unwrap()
        },
        SwitchReceiver{
            count: switch.clone(),
            receivers: receivers.try_into().unwrap()
        }
    )
}


#[cfg(test)]
mod tests{
    use crate::*;
    use super::*;
    use futures::{future, FutureExt};
    use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    use async_std::task;

    #[test]
    fn constructors(){
        let (sender, receiver) = unbounded::<u32, 10, false, false>();
        let (sender, receiver) = unbounded::<u32, 10, false, true>();
        let (sender, receiver) = unbounded::<u32, 10, true, false>();
        let (sender, receiver) = unbounded::<u32, 10, true, true>();
        let (sender, receiver) = bounded::<u32, 10, false, false>(10);
        let (sender, receiver) = bounded::<u32, 10, false, true>(10);
        let (sender, receiver) = bounded::<u32, 10, true, false>(10);
        let (sender, receiver) = bounded::<u32, 10, true, true>(10);
    }

    #[async_std::test]
    async fn unbounded_send_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 1, false, false>();
        sender.send(10).await?;
        assert_eq!(10, receiver.recv().await?);
        Ok(())
    }

    #[async_std::test]
    async fn unbounded_switch_send_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 2, true, false>();
        sender.switch_add(1).send(10).await?;
        assert_eq!(err::recv::TryRecvError::Empty, receiver.try_recv().err().unwrap());
        Ok(())
    }

    #[async_std::test]
    async fn unbounded_send_switch_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 2, false, true>();
        sender.send(10).await?;
        assert_eq!(10, receiver.switch_add(1).try_recv().ok().unwrap());
        assert_eq!(err::recv::TryRecvError::Empty, receiver.try_recv().err().unwrap());
        Ok(())
    }

    #[async_std::test]
    async fn unbounded_switch_send_switch_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 2, true, true>();
        // This switches both the sender and receiver, and sends 10 on the previous channel
        sender.switch_add(1).send(10).await?;
        // This sends 20 on the current channel
        sender.send(20).await?;
        // This recieves 20 on the current channel, and switches back to the previous one
        assert_eq!(20, receiver.switch_add(1).try_recv().ok().unwrap());
        // This recieves the 10 we originally sent
        assert_eq!(10, receiver.try_recv().ok().unwrap());
        Ok(())
    }

    async fn add_switch_loop(value: &mut usize, add: Result<usize, err::recv::RecvError>, add_receiver: &SwitchReceiver<usize, 2, true>){
        if let Ok(add) = add{
            *value += add
        }
        while let Ok(add) = add_receiver.switch_xor(1).recv().await{
            *value += add
        }
    }

    async fn parallel_switch_loop(value: &AtomicUsize, para: Result<(), err::recv::RecvError>, para_receiver: &SwitchReceiver<(), 2, true>){
        if let Ok(_) = para{
            value.fetch_add(1, Ordering::SeqCst);
        }
        for _ in para_receiver
            .switch_xor(1)
            .into_iter(){
                value.fetch_add(1, Ordering::SeqCst);
            }
    }

    async fn parallel_use_case(add_target: usize, par_target: usize) -> Result<(), Box<dyn std::error::Error>>{
        let (add_sender, add_receiver) = unbounded::<_, 2, false, true>();
        let (para_sender, para_receiver) = unbounded::<_, 2, false, true>();

        let add_target = 1000;
        let par_target = 1000000;

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
        let mut para_adder_value = AtomicUsize::new(0);

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
        assert_eq!(para_adder_value.load(Ordering::SeqCst), par_target);
        Ok(())
    }

    #[async_std::test]
    async fn test_para_use_case() -> Result<(), Box<dyn std::error::Error>>{
        parallel_use_case(1000, 1000000000).await
    }
}
