use thiserror::Error;
use tokio::task::JoinError;

#[derive(Error, Debug)]
pub enum Error{
    #[error("Tokio SendError: {0}")]
    SendError(String),

    #[error("CPU helper couldn't be contacted")]
    CpuAns,

    #[error("RAM helper couldn't be contacted")]
    RamAns,

    #[error("This thread panicked, Couldn't Join")]
    Joinerror(#[from] JoinError)
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error{
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self{
        Error::SendError(err.to_string())
    }
}