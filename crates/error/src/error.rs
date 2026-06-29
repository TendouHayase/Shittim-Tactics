use std::{error::Error as StdError, fmt::Display, io};

use crate::Error::{
    ParsingDataError, ParsingEofError, ParsingIoError, ParsingSyntaxError, UnknownError,
};

#[derive(Debug)]
pub enum Error {
    ReadFileError(String),
    ParsingSyntaxError(String),
    ParsingDataError(String),
    ParsingEofError(String),
    ParsingIoError(String),
    UnknownError(String),
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
        Error::ReadFileError(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        if value.is_syntax() {
            ParsingSyntaxError(value.to_string())
        } else if value.is_data() {
            ParsingDataError(value.to_string())
        } else if value.is_eof() {
            ParsingEofError(value.to_string())
        } else if value.is_io() {
            ParsingIoError(value.to_string())
        } else {
            UnknownError(value.to_string())
        }
    }
}
