use core::fmt;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T>{
    Full(T),
    Closed(T),
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

impl<T: fmt::Debug> std::error::Error for SendError<T> {}

impl<T> std::fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sending into a closed channel")
    }
}