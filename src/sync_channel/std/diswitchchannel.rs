use super::*;
use crate::Switcher;

pub type DiSwitchSender<T> = SwitchSender<T, 2, false>;
pub type DiSwitchSyncSender<T> = SwitchSyncSender<T, 2, false>;
pub type DiSwitchReceiver<T> = SwitchReceiver<T, 2, true>;

pub fn dibounded<T, const N: usize, const S: bool, const P: bool>(cap: usize) -> (DiSwitchSyncSender<T>, DiSwitchReceiver<T>){
    bounded::<T, 2, false, true>(cap)
}

pub fn diunbounded<T, const N: usize, const S: bool, const P: bool>() -> (DiSwitchSender<T>, DiSwitchReceiver<T>){
    unbounded::<T, 2, false, true>()
}

impl<T: 'static> DiSwitchReceiver<T>{
    pub fn switch(&self) -> SwitchReceiverGuard<'_, T>{
        self.switch_xor(1)
    }
}