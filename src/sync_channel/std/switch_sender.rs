use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::sync::mpsc::Sender;
use crate::{PERMITTED};
use crate::{Switcher, err::send::{SendError, TrySendError}};

#[derive(Clone)]
pub struct SwitchSender<T, const N: usize, const P: bool>{
    pub(crate) count: Arc<AtomicUsize>,
    pub(crate) senders: [Sender<T>; N],
}

impl<T, const N: usize, const P: bool> SwitchSender<T, N, P>{
    pub async fn send(&'_ self, msg: T) -> Result<(), SendError<T>>{
        Ok(self.senders[self.count.load(Ordering::SeqCst) % N].send(msg)?)
    }
}

#[derive(Clone)]
pub struct SwitchSenderGuard<'a, T>{
    sender: &'a Sender<T>
}

impl<'a, T> SwitchSenderGuard<'a, T>{
    pub async fn send(&'_ self, msg: T) -> Result<(), SendError<T>>{
        Ok(self.sender.send(msg)?)
    }
}

impl<'a, T: 'static, const N: usize> Switcher<'a, T> for SwitchSender<T, N, PERMITTED>{
    type Output = SwitchSenderGuard<'a, T>;

    fn switch_add(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_add(val, Ordering::SeqCst)]
        }
    }

    fn switch_and(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_and(val, Ordering::SeqCst)]
        }
    }

    fn switch_max(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_max(val, Ordering::SeqCst)]
        }
    }

    fn switch_min(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_min(val, Ordering::SeqCst)]
        }
    }

    fn switch_nand(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_nand(val, Ordering::SeqCst)]
        }
    }

    fn switch_or(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_or(val, Ordering::SeqCst)]
        }
    }

    fn switch_sub(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_sub(val, Ordering::SeqCst)]
        }
    }

    fn switch_xor(&self, val: usize) -> SwitchSenderGuard<'_, T>{
        SwitchSenderGuard{
            sender: &self.senders[self.count.fetch_xor(val, Ordering::SeqCst)]
        }
    }
}
