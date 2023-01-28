use failure_derive::Fail;
use std::{io, result, string};

#[derive(Fail, Debug)]
pub enum KvError {
    #[fail(display = "{}", _0)]
    Message(String), // wrap of RESPError(String)
    #[fail(display = "IO error: {}", _0)]
    IoError(#[cause] io::Error),
    #[fail(display = "Key {} not found.", _0)]
    KeyNotFound(String),
    #[fail(display = "Json error: {}", _0)]
    JsonError(#[cause] serde_json::Error),
    #[fail(display = "Resp error: {}", _0)]
    RESPError(#[cause] serde_resp::Error),
    #[fail(display = "Unexpected Command Type: {}", _0)]
    UnexpectedCmdType(String),
    #[fail(display = "Unknown command")]
    UnknownCommand,
    #[fail(display = "Missing arguments")]
    MissingArguments,
    #[fail(display = "Sled error: {}", _0)]
    SledError(sled::Error),
    #[fail(display = "From utf8 error: {}", _0)]
    FromUtf8Error(string::FromUtf8Error)
}

impl KvError {
    pub fn kind(&self) -> KvErrorKind {
        match *self {
            KvError::IoError(_) => KvErrorKind::IoError,
            KvError::KeyNotFound(_) => KvErrorKind::KeyNotFound,
            KvError::JsonError(_) => KvErrorKind::JsonError,
            KvError::RESPError(_) => KvErrorKind::RESPError,
            KvError::UnexpectedCmdType(_) => KvErrorKind::UnexpectedCmdType,
            KvError::UnknownCommand => KvErrorKind::UnknownCommand,
            KvError::MissingArguments => KvErrorKind::MissingArguments,
            KvError::FromUtf8Error(_) => KvErrorKind::FromUtf8Error,
            KvError::Message(_) => KvErrorKind::Message,
            KvError::SledError(_) => KvErrorKind::SledError
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

impl From<serde_resp::Error> for KvError {
    fn from(value: serde_resp::Error) -> Self {
        KvError::RESPError(value)
    }
}

impl From<sled::Error> for KvError {
    fn from(value: sled::Error) -> Self {
        KvError::SledError(value)
    }
}

impl From<string::FromUtf8Error> for KvError {
    fn from(value: string::FromUtf8Error) -> Self {
        KvError::FromUtf8Error(value)
    }
}

pub type Result<T> = result::Result<T, KvError>;

#[derive(Eq, PartialEq)]
pub enum KvErrorKind {
    Message,
    IoError,
    KeyNotFound,
    JsonError,
    RESPError,
    UnexpectedCmdType,
    UnknownCommand,
    MissingArguments,
    FromUtf8Error,
    SledError
}
