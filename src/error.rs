///! Error types
use std::io;
use crate::protocol::{ResponseCode, CommandType};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    /// Error communicating with the reader
    Io(io::Error),
    /// Error communicating with the tag - usually transient and may be retried
    Communication(ResponseCode),
    /// Error returned from the tag
    Protocol(ResponseCode),
    /// Incorrect parameters, or internal library error
    Program(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Error {
        Error::Program(e)
    }
}

impl From<bitreader::BitReaderError> for Error {
    fn from(e: bitreader::BitReaderError) -> Error {
        Error::Program(format!("Bitwise parsing error: {:?}", e))
    }
}

impl From<num_enum::TryFromPrimitiveError<ResponseCode>> for Error {
    fn from(e: num_enum::TryFromPrimitiveError<ResponseCode>) -> Error {
        Error::Program(format!("Error parsing response code: {:?}", e))
    }
}

impl From<num_enum::TryFromPrimitiveError<CommandType>> for Error {
    fn from(e: num_enum::TryFromPrimitiveError<CommandType>) -> Error {
        Error::Program(format!("Error parsing command type: {:?}", e))
    }
}

impl From<ResponseCode> for Error {
    fn from(e: ResponseCode) -> Error {
        match e {
            other => Error::Program(format!("Invalid status response: {:?}", other))
        }
    }
}
