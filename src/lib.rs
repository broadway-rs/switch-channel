pub mod err;
mod async_switch_channel;

pub use async_switch_channel::{SwitchSender, SwitchReceiver, Switcher, bounded, unbounded};