///! Error types
use std::io;
use crate::ResponseCode;

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

impl From<ResponseCode> for Error {
    fn from(e: ResponseCode) -> Error {
        match e {
            /*
            ResponseStatus::PoorCommunication => Error::Communication(e),
            ResponseStatus::NoTags => Error::Communication(e),

            ResponseStatus::AccessPasswordError => Error::Protocol(e),
            ResponseStatus::KillTagError => Error::Protocol(e),
            ResponseStatus::KillPasswordZero => Error::Protocol(e),
            ResponseStatus::CommandNotSupported => Error::Protocol(e),

            ResponseStatus::WrongLength => Error::Program("Wrong command length".to_string()),
            ResponseStatus::IllegalCommand => Error::Program("Illegal command".to_string()),
            ResponseStatus::ParameterError => Error::Program("Parameter error".to_string()),
*/
            other => Error::Program(format!("Invalid status response: {:?}", other))
        }
    }
}
