#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError{
    Empty,
    Closed,
}

impl TryRecvError {
    /// Returns `true` if the channel is empty but not closed.
    pub fn is_empty(&self) -> bool {
        match self {
            TryRecvError::Empty => true,
            TryRecvError::Closed => false,
        }
    }

    /// Returns `true` if the channel is empty and closed.
    pub fn is_closed(&self) -> bool {
        match self {
            TryRecvError::Empty => false,
            TryRecvError::Closed => true,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct RecvError;

impl std::error::Error for RecvError {}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "receiving from an empty and closed channel")
    }
}