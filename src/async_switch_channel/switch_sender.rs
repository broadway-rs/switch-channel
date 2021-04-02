use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use async_std::channel::Sender;
use super::{Switcher, PERMITTED};
use crate::err::send_err::{SendError, TrySendError};

#[derive(Clone)]
pub struct SwitchSender<T, const N: usize, const P: bool>{
    pub(crate) count: Arc<AtomicUsize>,
    pub(crate) senders: [Sender<T>; N],
}

impl<T, const N: usize, const P: bool> SwitchSender<T, N, P>{
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>>{
        Ok(self.senders[self.count.load(Ordering::SeqCst) % N].try_send(msg)?)
    }

    pub async fn send(&'_ self, msg: T) -> Result<(), SendError<T>>{
        Ok(self.senders[self.count.load(Ordering::SeqCst) % N].send(msg).await?)
    }

    pub fn close(&self) -> bool{
        // Any number of threads could call close,
        // but only one will get a false value from
        // the first call of reciever close.
        for reciever in self.senders.iter(){
            if reciever.close(){
                return true;
            }
        }
        return false
    }

    pub fn is_closed(&self) -> bool{
        self.senders[0].is_closed()
    }

    pub fn is_full(&self) -> bool{
        self.senders[self.count.load(Ordering::SeqCst) % N].is_full()
    }

    pub fn len(&self) -> usize{
        self.senders[self.count.load(Ordering::SeqCst) % N].len()
    }

    pub fn capacity(&self) -> Option<usize>{
        self.senders[0].capacity()
    }

    /// Returns the number of senders for the channel.
    pub fn sender_count(&self) -> usize{
        self.senders[0].sender_count()
    }

    /// Returns the number of receivers for the channel.
    pub fn receiver_count(&self) -> usize{
        self.senders[0].receiver_count()
    }
}

impl<T, const N: usize> Switcher for SwitchSender<T, N, PERMITTED>{
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