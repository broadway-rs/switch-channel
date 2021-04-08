use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use async_std::channel::Receiver;
use crate::{PERMITTED};
use crate::{Switcher, err::recv::{RecvError, TryRecvError}};

#[derive(Clone)]
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
    pub async fn recv(&'_ self) -> Result<T, RecvError>{
        Ok(self.receivers[self.count.load(Ordering::SeqCst) % N].recv().await?)
    }

    /// Close all the channels.
    pub fn close(&self) -> bool{
        // Any number of threads could call close,
        // but only one will get a false value from
        // the first call of receiver close.
        for receiver in self.receivers.iter(){
            if receiver.close(){
                return true;
            }
        }
        return false
    }

    /// Checks if all the channels have been closed.
    pub fn is_closed(&self) -> bool{
        self.receivers[0].is_closed()
    }

    /// Check if the activate channel is empty.
    pub fn is_empty(&self) -> bool{
        self.receivers[self.count.load(Ordering::SeqCst) % N].is_empty()
    }

    /// Check if the activate channel is full, unbounded channels are never full.
    pub fn is_full(&self) -> bool{
        self.receivers[self.count.load(Ordering::SeqCst) % N].is_full()
    }

    pub fn capacity(&self) -> Option<usize>{
        // These are only allowed be constructed with uniform capacity,
        // so you can just get the capacity from any instance.
        self.receivers[0].capacity()
    }

    /// Returns the number of senders for the channel.
    pub fn sender_count(&self) -> usize{
        self.receivers[0].sender_count()
    }

    /// Returns the number of receivers for the channel.
    pub fn receiver_count(&self) -> usize{
        self.receivers[0].receiver_count()
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
    pub async fn recv(&'_ self) -> Result<T, RecvError>{
        Ok(self.receiver.recv().await?)
    }

    /// Checks if all the channels have been closed.
    pub fn is_closed(&self) -> bool{
        self.receiver.is_closed()
    }

    /// Check if the activate channel is empty.
    pub fn is_empty(&self) -> bool{
        self.receiver.is_empty()
    }

    /// Check if the activate channel is full, unbounded channels are never full.
    pub fn is_full(&self) -> bool{
        self.receiver.is_full()
    }

    pub fn capacity(&self) -> Option<usize>{
        // These are only allowed be constructed with uniform capacity,
        // so you can just get the capacity from any instance.
        self.receiver.capacity()
    }

    /// Returns the number of senders for the channel.
    pub fn sender_count(&self) -> usize{
        self.receiver.sender_count()
    }

    /// Returns the number of receivers for the channel.
    pub fn receiver_count(&self) -> usize{
        self.receiver.receiver_count()
    }
} 

impl<'a, T> std::iter::IntoIterator for SwitchReceiverGuard<'a, T>{
    type Item = T;
    type IntoIter = SwitchReceiverGuardIterator<'a, T>;
    
    fn into_iter(self) -> <Self as std::iter::IntoIterator>::IntoIter { 
        SwitchReceiverGuardIterator{
            receiver: self.receiver
        }
    }
}

pub struct SwitchReceiverGuardIterator<'a, T>{
    receiver: &'a Receiver<T>,
}

impl<'a, T> std::iter::Iterator for SwitchReceiverGuardIterator<'a, T>{

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

impl From<async_std::channel::TryRecvError> for TryRecvError{
    fn from(err: async_std::channel::TryRecvError) -> Self { 
        match err{
            async_std::channel::TryRecvError::Empty => Self::Empty,
            async_std::channel::TryRecvError::Closed => Self::Closed,
        }
    }
}

impl From<async_std::channel::RecvError> for RecvError{
    fn from(_: async_std::channel::RecvError) -> Self { 
        Self
    }
}