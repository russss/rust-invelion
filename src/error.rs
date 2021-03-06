///! Error types
use std::io;
use failure::Fail;
use crate::protocol::{ResponseCode, CommandType};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display="Reader I/O error")]
    Io(#[fail(cause)] io::Error),
    #[fail(display="Transient error communicating with tag: {:?}", _0)]
    Communication(ResponseCode),
    #[fail(display="Error returned from tag: {:?}", _0)]
    Protocol(ResponseCode),
    #[fail(display="Program error: {}", _0)]
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
