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

impl From<async_std::channel::TryRecvError> for TryRecvError{
    fn from(err: async_std::channel::TryRecvError) -> Self { 
        match err{
            async_std::channel::TryRecvError::Empty => Self::Empty,
            async_std::channel::TryRecvError::Closed => Self::Closed,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct RecvError;

impl From<async_std::channel::RecvError> for RecvError{
    fn from(_: async_std::channel::RecvError) -> Self { 
        Self
    }
}

impl std::error::Error for RecvError {}

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "receiving from an empty and closed channel")
    }
}