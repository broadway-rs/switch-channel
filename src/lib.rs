//! A switch channel is a way of grouping channels together in such a way,
//! that a struct using multiple switch channels is able to read from the channels,
//! without having to worry about starvation.
//! # Example
//! ```
//! use switch_channel::async_channel::;
//! use futures::{future, FutureExt};
//! use async_std::task;
//! 
//! async fn add_switch_loop(value: &mut i32, add: Result<i32, err::recv::RecvError>, add_receiver: &SwitchReceiver<i32, 2, true>){
//!     if let Ok(add) = add{
//!         *value += add
//!     }
//!     while let Ok(add) = add_receiver.switch_xor(1).recv().await{
//!         *value += add
//!     }
//! }
//!
//! async fn sub_switch_loop(value: &mut i32, sub: Result<i32, err::recv::RecvError>, sub_receiver: &SwitchReceiver<i32, 2, true>){
//!     if let Ok(sub) = sub{
//!         *value -= sub
//!     }
//!     while let Ok(sub) = sub_receiver.switch_xor(1).recv().await{
//!         *value -= sub;
//!     }
//! }
//!
//! #[async_std::test]
//! async fn dumb_use_case() -> Result<(), Box<dyn std::error::Error>>{
//!     let (add_sender, add_receiver) = unbounded::<i32, 2, false, true>();
//!     let (sub_sender, sub_receiver) = unbounded::<i32, 2, false, true>();
//! 
//!     let handle = task::spawn(async move {
//!         for i in 0i32..100000{
//!             let _ = add_sender.send(1).await;
//!             let _ = sub_sender.send(1).await;
//!         };
//!         add_sender.close();
//!         sub_sender.close();
//!     });
//! 
//!     // Not mandatory, just want to let some messages build up
//!     task::sleep(std::time::Duration::from_secs(3)).await;
//! 
//!     let mut value: i32 = 0;
//! 
//!     let mut add_recv_fut = None;
//!     let mut sub_recv_fut = None;
//! 
//!     loop {
//!         if let None = add_recv_fut{
//!             if !add_receiver.is_empty() || !add_receiver.is_closed(){
//!                 add_recv_fut = Some(Box::pin(add_receiver.recv()));
//!             }
//!         }
//!         if let None = sub_recv_fut {
//!             if !sub_receiver.is_empty() || !sub_receiver.is_closed(){
//!                 sub_recv_fut = Some(Box::pin(sub_receiver.recv()));
//!             }
//!         }
//! 
//!         let select_fut = match (add_recv_fut.take(), sub_recv_fut.take()){
//!             (Some(add_fut), Some(sub_fut)) =>{
//!                 match future::select(add_fut, sub_fut).await{
//!                     future::Either::Left((add, sub_fut)) => {
//!                         add_switch_loop(&mut value, add, &add_receiver).await;
//!                         sub_recv_fut = Some(sub_fut);
//!                     },
//!                     future::Either::Right((sub, add_fut)) => {
//!                         sub_switch_loop(&mut value, sub, &sub_receiver).await;
//!                         add_recv_fut = Some(add_fut);
//!                     },
//!                 }
//!             },
//!             (Some(add_fut), None) => add_switch_loop(&mut value, add_fut.await, &add_receiver).await,
//!             (None, Some(sub_fut)) => sub_switch_loop(&mut value, sub_fut.await, &sub_receiver).await,
//!             (None, None) => break,
//!         };
//!     };
//! 
//!     handle.await;
//!     assert_eq!(value, 0);
//!     Ok(())
//! }
//! ```

pub mod err;
pub mod async_channel;
pub mod sync_channel;

pub trait Switcher<'a, T>{
    type Output;

    fn switch_add(&'a self, val: usize) -> Self::Output;
    fn switch_and(&'a self, val: usize) -> Self::Output;
    fn switch_max(&'a self, val: usize) -> Self::Output;
    fn switch_min(&'a self, val: usize) -> Self::Output;
    fn switch_nand(&'a self, val: usize) -> Self::Output;
    fn switch_or(&'a self, val: usize) -> Self::Output;
    fn switch_sub(&'a self, val: usize) -> Self::Output;
    fn switch_xor(&'a self, val: usize) -> Self::Output;
}

pub const PERMITTED: bool = true;
pub const NOT_PERMITTED: bool = false;