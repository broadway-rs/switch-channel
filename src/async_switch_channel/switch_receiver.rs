use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use async_std::channel::Receiver;
use super::{Switcher, PERMITTED};
use crate::err::recv_err::{RecvError, TryRecvError};

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
    pub async fn rev(&'_ self) -> Result<T, RecvError>{
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
} 

impl<T, const N: usize> Switcher for SwitchReceiver<T, N, PERMITTED>{
    fn switch_add(&self, val: usize) -> usize{
        self.count.fetch_add(val, Ordering::SeqCst)
    }

    fn switch_and(&self, val: usize) -> usize{
        self.count.fetch_and(val, Ordering::SeqCst)
    }

    fn switch_max(&self, val: usize) -> usize{
        self.count.fetch_max(val, Ordering::SeqCst)
    }

    fn switch_min(&self, val: usize) -> usize{
        self.count.fetch_min(val, Ordering::SeqCst)
    }

    fn switch_nand(&self, val: usize) -> usize{
        self.count.fetch_nand(val, Ordering::SeqCst)
    }

    fn switch_or(&self, val: usize) -> usize{
        self.count.fetch_or(val, Ordering::SeqCst)
    }

    fn switch_sub(&self, val: usize) -> usize{
        self.count.fetch_sub(val, Ordering::SeqCst)
    }

    fn switch_update<F>(&self, f: F) -> Result<usize, usize>
        where F: FnMut(usize) -> Option<usize>{
        self.count.fetch_update(Ordering::SeqCst, Ordering::SeqCst, f)
    }

    fn switch_xor(&self, val: usize) -> usize{
        self.count.fetch_xor(val, Ordering::SeqCst)
    }
}