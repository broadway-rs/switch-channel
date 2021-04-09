//! A switch channel is a way of grouping channels together in such a way,
//! that a struct using multiple switch channels is able to read from the channels,
//! without having to worry about starvation.

pub mod err;
pub mod async_channel;
pub mod sync_channel;

pub trait Switcher<'a, T>{
    type Output;

    fn switch_add(&'a self, val: usize) -> Self::Output;
    fn switch_and(&'a self, val: usize) -> Self::Output;
    fn switch_max(&'a self, val: usize) -> Self::Output;
    fn switch_min(&'a self, val: usize) -> Self::Output;
    fn switch_nand(&'a self, val: usize) -> Self::Output;
    fn switch_or(&'a self, val: usize) -> Self::Output;
    fn switch_sub(&'a self, val: usize) -> Self::Output;
    fn switch_xor(&'a self, val: usize) -> Self::Output;
}

pub const PERMITTED: bool = true;
pub const NOT_PERMITTED: bool = false;