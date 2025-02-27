use crate::error::Error::{ErrNo, HidrawDeviceNotFound, Problem};
use nix::errno::Errno;
use std::ffi::{FromBytesUntilNulError, NulError};
use std::fmt::{Display, Formatter};
use std::io;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum Error {
    Problem(String),
    ErrNo(Errno),
    HidrawDeviceNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Problem(msg) => write!(f, "Problem: {}", msg),
            ErrNo(errno) => write!(f, "Errno: {}", errno),
            HidrawDeviceNotFound => write!(f, "Hidraw device not found"),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Problem(value.to_string())
    }
}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Problem(value.to_string())
    }
}

impl From<FromBytesUntilNulError> for Error {
    fn from(value: FromBytesUntilNulError) -> Self {
        Problem(value.to_string())
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Problem(value.to_string())
    }
}

impl From<Errno> for Error {
    fn from(value: Errno) -> Self {
        ErrNo(value)
    }
}
