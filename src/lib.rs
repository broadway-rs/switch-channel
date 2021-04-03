//! A switch channel is a way of grouping channels together in such a way,
//! that a struct using multiple switch channels is able to read from the channels,
//! without having to worry about starvation.
//! # Example
//! ```
//! # #[async_std::test]
//! # async fn unbounded_switch_send_switch_receive() -> Result<(), Box<dyn std::error::Error>>{
//! let (sender, receiver) = unbounded::<u32, 2, true, true>();
//! // This switches both the sender and receiver, and sends 10 on the previous channel
//! sender.switch_add(1).send(10).await?;
//! // This sends 20 on the current channel
//! sender.send(20).await?;
//! // This recieves 20 on the current channel, and switches back to the previous one
//! assert_eq!(20, receiver.switch_add(1).try_recv().ok().unwrap());
//! // This recieves the 10 we originally sent
//! assert_eq!(10, receiver.try_recv().ok().unwrap());
//! Ok(())
//! # }
//! ```

pub mod err;
mod async_switch_channel;

pub use async_switch_channel::{SwitchSender, SwitchReceiver, SendSwitcher, ReceiveSwitcher, bounded, unbounded};
