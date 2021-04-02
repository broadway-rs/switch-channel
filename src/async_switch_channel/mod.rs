mod switch_receiver;
mod switch_sender;

use std::convert::TryInto;
use core::sync::atomic::AtomicUsize;
use std::sync::Arc;
use core::iter::repeat_with;
pub use switch_receiver::{SwitchReceiver};
pub use switch_sender::{SwitchSender};

#[derive(Eq, PartialEq)]
pub enum SwitchPermission{
    Permitted = 0,
    NotPermitted = 1
}

pub const PERMITTED: bool = true;
pub const NOT_PERMITTED: bool = false;

pub trait Switcher{
    fn switch_add(&self, val: usize) -> usize;
    fn switch_and(&self, val: usize) -> usize;
    fn switch_max(&self, val: usize) -> usize;
    fn switch_min(&self, val: usize) -> usize;
    fn switch_nand(&self, val: usize) -> usize;
    fn switch_or(&self, val: usize) -> usize;
    fn switch_sub(&self, val: usize) -> usize;
    fn switch_update<F>(&self, f: F) -> Result<usize, usize>
        where F: FnMut(usize) -> Option<usize>;
    fn switch_xor(&self, val: usize) -> usize;
}

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
    
    let mut senders = Vec::with_capacity(N);
    let mut receivers = Vec::with_capacity(N);
    for i in 0..N{
        let (sender, receiver) = unbounded();
        senders.push(sender);
        receivers.push(receiver);
    }

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