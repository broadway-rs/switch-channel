use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::sync::mpsc::Receiver;
use crate::{PERMITTED};
use crate::{Switcher, err::recv::{RecvError, TryRecvError}};

pub struct SwitchReceiver<T, const N: usize, const P: bool>{
    pub(crate) count: Arc<AtomicUsize>,
    pub(crate) receivers: [Receiver<T>; N],
}

impl<T, const N: usize, const P: bool> SwitchReceiver<T, N, P>{
    /// Try to receive from the activate channel.
    pub fn try_recv(&self) -> Result<T, TryRecvError>{
        Ok(self.receivers[self.count.load(Ordering::SeqCst) % N].try_recv()?)
    }

    /// receive from the activate channel.
    pub fn recv(&self) -> Result<T, RecvError>{
        Ok(self.receivers[self.count.load(Ordering::SeqCst) % N].recv()?)
    }

    pub fn iter(&self) -> SwitchReceiverGuardIterator<'_, T>{
        SwitchReceiverGuardIterator{
            receiver: &self.receivers[self.count.load(Ordering::SeqCst) % N]
        }
    }

    pub fn try_iter(&self) -> SwitchReceiverGuardTryIterator<'_, T>{
        SwitchReceiverGuardTryIterator{
            receiver: &self.receivers[self.count.load(Ordering::SeqCst) % N]
        }
    }

    pub fn get_guard(&self) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.load(Ordering::SeqCst) % N]
        }
    }
} 

#[derive(Clone)]
pub struct SwitchReceiverGuard<'a, T>{
    receiver: &'a Receiver<T>
}

impl<'a, T> SwitchReceiverGuard<'a, T>{
    /// Try to receive from the activate channel.
    pub fn try_recv(&self) -> Result<T, TryRecvError>{
        Ok(self.receiver.try_recv()?)
    }

    /// receive from the activate channel.
    pub fn recv(&'_ self) -> Result<T, RecvError>{
        Ok(self.receiver.recv()?)
    }

    pub fn iter(&self) -> SwitchReceiverGuardIterator<'_, T>{
        SwitchReceiverGuardIterator{
            receiver: &self.receiver
        }
    }

    pub fn try_iter(&self) -> SwitchReceiverGuardTryIterator<'_, T>{
        SwitchReceiverGuardTryIterator{
            receiver: &self.receiver
        }
    }
} 

pub struct SwitchReceiverGuardIterator<'a, T>{
    receiver: &'a Receiver<T>,
}

impl<'a, T> std::iter::Iterator for SwitchReceiverGuardIterator<'a, T>{

    type Item = T;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> { 
        self.receiver.recv().ok()
    }
}

pub struct SwitchReceiverGuardTryIterator<'a, T>{
    receiver: &'a Receiver<T>,
}

impl<'a, T> std::iter::Iterator for SwitchReceiverGuardTryIterator<'a, T>{

    type Item = T;
    fn next(&mut self) -> std::option::Option<<Self as std::iter::Iterator>::Item> { 
        self.receiver.try_recv().ok()
    }
}

impl<'a, T: 'static, const N: usize> Switcher<'a, T> for SwitchReceiver<T, N, PERMITTED>{
    type Output = SwitchReceiverGuard<'a, T>;

    fn switch_add(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_add(val, Ordering::SeqCst)]
        }
    }

    fn switch_and(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_and(val, Ordering::SeqCst)]
        }
    }

    fn switch_max(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_max(val, Ordering::SeqCst)]
        }
    }

    fn switch_min(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_min(val, Ordering::SeqCst)]
        }
    }

    fn switch_nand(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_nand(val, Ordering::SeqCst)]
        }
    }

    fn switch_or(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_or(val, Ordering::SeqCst)]
        }
    }

    fn switch_sub(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_sub(val, Ordering::SeqCst)]
        }
    }

    fn switch_xor(&self, val: usize) -> SwitchReceiverGuard<'_, T>{
        SwitchReceiverGuard{
            receiver: &self.receivers[self.count.fetch_xor(val, Ordering::SeqCst)]
        }
    }
}

impl From<std::sync::mpsc::TryRecvError> for TryRecvError{
    fn from(err: std::sync::mpsc::TryRecvError) -> Self { 
        match err{
            std::sync::mpsc::TryRecvError::Empty => Self::Empty,
            std::sync::mpsc::TryRecvError::Disconnected => Self::Closed,
        }
    }
}

impl From<std::sync::mpsc::RecvError> for RecvError{
    fn from(_: std::sync::mpsc::RecvError) -> Self { 
        Self
    }
}