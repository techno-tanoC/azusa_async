use std::{io, ffi};
use tokio::task;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ReqwestError(reqwest::Error),
    StdIoError(io::Error),
    JoinError(task::JoinError),
    CStringNulError(ffi::NulError),
    // TokioIoError(tokio::io::Error),
    HttpStatusError,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::ReqwestError(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::StdIoError(err)
    }
}

impl From<task::JoinError> for Error {
    fn from(err: task::JoinError) -> Self {
        Error::JoinError(err)
    }
}

impl From<ffi::NulError> for Error {
    fn from(err: ffi::NulError) -> Self {
        Error::CStringNulError(err)
    }
}
