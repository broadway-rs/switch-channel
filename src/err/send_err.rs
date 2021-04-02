use core::fmt;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T>{
    Full(T),
    Closed(T),
}


impl<T> From<async_std::channel::TrySendError<T>> for TrySendError<T>{
    fn from(err: async_std::channel::TrySendError<T>) -> Self { 
        match err{
            async_std::channel::TrySendError::Full(t) => Self::Full(t),
            async_std::channel::TrySendError::Closed(t) => Self::Closed(t),
        }
    }
}

impl<T: fmt::Debug> std::error::Error for TrySendError<T> {}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => write!(f, "Full(..)"),
            TrySendError::Closed(..) => write!(f, "Closed(..)"),
        }
    }
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => write!(f, "sending into a full channel"),
            TrySendError::Closed(..) => write!(f, "sending into a closed channel"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct SendError<T>(pub T);

impl<T> From<async_std::channel::SendError<T>> for SendError<T>{
    fn from(err: async_std::channel::SendError<T>) -> Self { 
        Self(err.0)
    }
}

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sending into a closed channel")
    }
}