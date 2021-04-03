mod switch_receiver;
mod switch_sender;

use std::convert::TryInto;
use core::sync::atomic::AtomicUsize;
use std::sync::Arc;
use core::iter::repeat_with;
pub use switch_receiver::{SwitchReceiver, ReceiveSwitcher};
pub use switch_sender::{SwitchSender, SendSwitcher};

pub const PERMITTED: bool = true;
pub const NOT_PERMITTED: bool = false;

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
    
    let (senders, receivers): (Vec<Sender<T>>, Vec<Receiver<T>>) = repeat_with(|| unbounded()).take(N).unzip();

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

#[cfg(test)]
mod tests{
    use crate::*;

    #[test]
    fn constructors(){
        let (sender, receiver) = unbounded::<u32, 10, false, false>();
        let (sender, receiver) = unbounded::<u32, 10, false, true>();
        let (sender, receiver) = unbounded::<u32, 10, true, false>();
        let (sender, receiver) = unbounded::<u32, 10, true, true>();
        let (sender, receiver) = bounded::<u32, 10, false, false>(10);
        let (sender, receiver) = bounded::<u32, 10, false, true>(10);
        let (sender, receiver) = bounded::<u32, 10, true, false>(10);
        let (sender, receiver) = bounded::<u32, 10, true, true>(10);
    }

    #[async_std::test]
    async fn unbounded_send_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 1, false, false>();
        sender.send(10).await?;
        assert_eq!(10, receiver.recv().await?);
        Ok(())
    }

    #[async_std::test]
    async fn unbounded_switch_send_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 2, true, false>();
        sender.switch_add(1).send(10).await?;
        assert_eq!(err::recv::TryRecvError::Empty, receiver.try_recv().err().unwrap());
        Ok(())
    }

    #[async_std::test]
    async fn unbounded_send_switch_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 2, false, true>();
        sender.send(10).await?;
        assert_eq!(10, receiver.switch_add(1).try_recv().ok().unwrap());
        assert_eq!(err::recv::TryRecvError::Empty, receiver.try_recv().err().unwrap());
        Ok(())
    }

    #[async_std::test]
    async fn unbounded_switch_send_switch_receive() -> Result<(), Box<dyn std::error::Error>>{
        let (sender, receiver) = unbounded::<u32, 2, true, true>();
        // This switches both the sender and receiver, and sends 10 on the previous channel
        sender.switch_add(1).send(10).await?;
        // This sends 20 on the current channel
        sender.send(20).await?;
        // This recieves 20 on the current channel, and switches back to the previous one
        assert_eq!(20, receiver.switch_add(1).try_recv().ok().unwrap());
        // This recieves the 10 we originally sent
        assert_eq!(10, receiver.try_recv().ok().unwrap());
        Ok(())
    }
}
