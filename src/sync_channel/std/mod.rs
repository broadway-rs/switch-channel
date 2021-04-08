mod switch_receiver;
mod switch_sender;
mod switch_sync_sender;

pub use switch_receiver::{SwitchReceiver, SwitchReceiverGuard, SwitchReceiverGuardIterator};
pub use switch_sender::{SwitchSender, SwitchSenderGuard};
pub use switch_sync_sender::{SwitchSyncSender, SwitchSyncSenderGuard};

use std::convert::TryInto;
use core::sync::atomic::AtomicUsize;
use std::sync::Arc;
use core::iter::repeat_with;
use crate::err::send::{SendError, TrySendError};

pub fn bounded<T, const N: usize, const S: bool, const P: bool>(cap: usize) -> (SwitchSyncSender<T, N, S>, SwitchReceiver<T, N, P>){
    use std::sync::mpsc::{sync_channel, SyncSender, Receiver};
    
    let (senders, receivers): (Vec<SyncSender<T>>, Vec<Receiver<T>>) = repeat_with(|| sync_channel(cap)).take(N).unzip();

    let switch = Arc::new(AtomicUsize::new(0));
    (
        SwitchSyncSender{
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
    use std::sync::mpsc::{channel, Sender, Receiver};
    
    let (senders, receivers): (Vec<Sender<T>>, Vec<Receiver<T>>) = repeat_with(|| channel()).take(N).unzip();

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


impl<T> From<std::sync::mpsc::SendError<T>> for SendError<T>{
    fn from(err: std::sync::mpsc::SendError<T>) -> Self { 
        Self(err.0)
    }
}

impl<T> From<std::sync::mpsc::TrySendError<T>> for TrySendError<T>{
    fn from(err: std::sync::mpsc::TrySendError<T>) -> Self { 
        match err{
            std::sync::mpsc::TrySendError::Full(t) => Self::Full(t),
            std::sync::mpsc::TrySendError::Disconnected(t) => Self::Closed(t),
        }
    }
}