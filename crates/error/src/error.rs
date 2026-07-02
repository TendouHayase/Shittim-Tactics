use std::{alloc::LayoutError, error::Error as StdError, fmt::Display, io, sync::MutexGuard};

use crate::Error::{
    InvalidData, InvalidSyntax, Io, MemoryAllocateFailed, Mutex, UnexpectedEof, Unknown,
};

#[derive(Debug)]
pub enum Error {
    ReadFileFailed(String),
    InvalidSyntax(String),
    InvalidData(String),
    UnexpectedEof(String),
    Io(String),
    Unknown(String),
    Mutex(String),
    MemoryAllocateFailed(String),
    UnallocatedMemory(String),
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn StdError> {
        None
    }

    fn description(&self) -> &str {
        "Error"
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error")
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::ReadFileFailed(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        if value.is_syntax() {
            InvalidSyntax(value.to_string())
        } else if value.is_data() {
            InvalidData(value.to_string())
        } else if value.is_eof() {
            UnexpectedEof(value.to_string())
        } else if value.is_io() {
            Io(value.to_string())
        } else {
            Unknown(value.to_string())
        }
    }
}

impl<T> From<std::sync::PoisonError<MutexGuard<'_, T>>> for Error {
    fn from(value: std::sync::PoisonError<MutexGuard<'_, T>>) -> Self {
        Mutex(value.to_string())
    }
}

impl From<LayoutError> for Error {
    fn from(value: LayoutError) -> Self {
        MemoryAllocateFailed(value.to_string())
    }
}
