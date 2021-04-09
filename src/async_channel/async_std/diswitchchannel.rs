use super::*;
use crate::Switcher;

pub type DiSwitchSender<T> = SwitchSender<T, 2, false>;
pub type DiSwitchReceiver<T> = SwitchReceiver<T, 2, true>;

pub fn dibounded<T>(cap: usize) -> (DiSwitchSender<T>, DiSwitchReceiver<T>){
    bounded::<T, 2, false, true>(cap)
}

pub fn diunbounded<T>() -> (DiSwitchSender<T>, DiSwitchReceiver<T>){
    unbounded::<T, 2, false, true>()
}

impl<T: 'static> DiSwitchReceiver<T>{
    pub fn switch(&self) -> SwitchReceiverGuard<'_, T>{
        self.switch_xor(1)
    }
}