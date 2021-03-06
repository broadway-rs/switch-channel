use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use async_std::channel::Sender;
use crate::{PERMITTED};
use crate::{Switcher, err::send::{SendError, TrySendError}};

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

#[derive(Clone)]
pub struct SwitchSenderGuard<'a, T>{
    sender: &'a Sender<T>
}

impl<'a, T> SwitchSenderGuard<'a, T>{
    pub fn try_send(&self, msg: T) -> Result<(), TrySendError<T>>{
        Ok(self.sender.try_send(msg)?)
    }

    pub async fn send(&'_ self, msg: T) -> Result<(), SendError<T>>{
        Ok(self.sender.send(msg).await?)
    }

    pub fn is_closed(&self) -> bool{
        self.sender.is_closed()
    }

    pub fn is_full(&self) -> bool{
        self.sender.is_full()
    }

    pub fn len(&self) -> usize{
        self.sender.len()
    }

    pub fn capacity(&self) -> Option<usize>{
        self.sender.capacity()
    }

    /// Returns the number of senders for the channel.
    pub fn sender_count(&self) -> usize{
        self.sender.sender_count()
    }

    /// Returns the number of receivers for the channel.
    pub fn receiver_count(&self) -> usize{
        self.sender.receiver_count()
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

impl<T> From<async_std::channel::SendError<T>> for SendError<T>{
    fn from(err: async_std::channel::SendError<T>) -> Self { 
        Self(err.0)
    }
}

impl<T> From<async_std::channel::TrySendError<T>> for TrySendError<T>{
    fn from(err: async_std::channel::TrySendError<T>) -> Self { 
        match err{
            async_std::channel::TrySendError::Full(t) => Self::Full(t),
            async_std::channel::TrySendError::Closed(t) => Self::Closed(t),
        }
    }
}

impl<T, const N: usize, const P: bool> Clone for SwitchSender<T, N, P>{
    fn clone(&self) -> Self{
        use std::convert::TryInto;
        let senders: Vec<Sender<T>> = self.senders.iter().map(|s| s.clone()).collect();
        Self{
            count: self.count.clone(),
            senders: senders.try_into().unwrap(),
        }
    }
}