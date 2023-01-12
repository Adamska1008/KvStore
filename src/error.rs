use failure_derive::Fail;
use std::{io, result};

#[derive(Fail, Debug)]
pub enum KvError {
    #[fail(display = "IO error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Key {} not found.", _0)]
    KeyNotFound(String),
    #[fail(display = "Json error: {}", _0)]
    JsonError(#[cause] serde_json::Error),
    #[fail(display = "Unexpected Command Type: {}", _0)]
    UnexpectedCmdType(String),
    #[fail(display = "Unknown command")]
    UnknownCommand,
    #[fail(display = "Missing arguments")]
    MissingArguments
}

impl KvError {
    pub fn kind(&self) -> KvErrorKind {
        match *self {
            KvError::IoError(_) => KvErrorKind::IoError,
            KvError::KeyNotFound(_) => KvErrorKind::KeyNotFound,
            KvError::JsonError(_) => KvErrorKind::JsonError,
            KvError::UnexpectedCmdType(_) => KvErrorKind::UnexpectedCmdType,
            KvError::UnknownCommand => KvErrorKind::UnknownCommand,
            KvError::MissingArguments => KvErrorKind::MissingArguments
        }
    }
}

impl From<io::Error> for KvError {
    fn from(value: io::Error) -> Self {
        KvError::IoError(value)
    }
}

impl From<serde_json::Error> for KvError {
    fn from(value: serde_json::Error) -> Self {
        KvError::JsonError(value)
    }
}

pub type Result<T> = result::Result<T, KvError>;

#[derive(Eq, PartialEq)]
pub enum KvErrorKind {
    IoError,
    KeyNotFound,
    JsonError,
    UnexpectedCmdType,
    UnknownCommand,
    MissingArguments
}
