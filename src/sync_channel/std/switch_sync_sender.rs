use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::sync::mpsc::SyncSender;
use crate::{PERMITTED};
use crate::{Switcher, err::send::{SendError, TrySendError}};

#[derive(Clone)]
pub struct SwitchSyncSender<T, const N: usize, const P: bool>{
    pub(crate) count: Arc<AtomicUsize>,
    pub(crate) senders: [SyncSender<T>; N],
}

impl<T, const N: usize, const P: bool> SwitchSyncSender<T, N, P>{
    pub fn send(&'_ self, msg: T) -> Result<(), SendError<T>>{
        Ok(self.senders[self.count.load(Ordering::SeqCst) % N].send(msg)?)
    }

    pub fn try_send(&'_ self, msg: T) -> Result<(), TrySendError<T>>{
        Ok(self.senders[self.count.load(Ordering::SeqCst) % N].try_send(msg)?)
    }
}

#[derive(Clone)]
pub struct SwitchSyncSenderGuard<'a, T>{
    sender: &'a SyncSender<T>
}

impl<'a, T> SwitchSyncSenderGuard<'a, T>{
    pub fn send(&'_ self, msg: T) -> Result<(), SendError<T>>{
        Ok(self.sender.send(msg)?)
    }

    pub fn try_send(&'_ self, msg: T) -> Result<(), TrySendError<T>>{
        Ok(self.sender.try_send(msg)?)
    }
}

impl<'a, T: 'static, const N: usize> Switcher<'a, T> for SwitchSyncSender<T, N, PERMITTED>{
    type Output = SwitchSyncSenderGuard<'a, T>;

    fn switch_add(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_add(val, Ordering::SeqCst)]
        }
    }

    fn switch_and(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_and(val, Ordering::SeqCst)]
        }
    }

    fn switch_max(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_max(val, Ordering::SeqCst)]
        }
    }

    fn switch_min(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_min(val, Ordering::SeqCst)]
        }
    }

    fn switch_nand(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_nand(val, Ordering::SeqCst)]
        }
    }

    fn switch_or(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_or(val, Ordering::SeqCst)]
        }
    }

    fn switch_sub(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_sub(val, Ordering::SeqCst)]
        }
    }

    fn switch_xor(&self, val: usize) -> SwitchSyncSenderGuard<'_, T>{
        SwitchSyncSenderGuard{
            sender: &self.senders[self.count.fetch_xor(val, Ordering::SeqCst)]
        }
    }
}