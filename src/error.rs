use crate::Tag;
use nom::error::{ErrorKind, ParseError};
use nom::IResult;
use std::str;
use std::string;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("Invalid Length")]
    InvalidLength,
    #[error("Invalid Tag")]
    InvalidTag,
    #[error("Unexpected Tag (expected: {0:?}")]
    UnexpectedTag(Tag),

    #[error("Indefinite length not allowed")]
    IndefiniteLengthUnexpected,

    #[error("DER object was expected to be constructed (and found to be primitive)")]
    ConstructExpected,
    #[error("DER object was expected to be primitive (and found to be constructed)")]
    ConstructUnexpected,

    #[error("Integer too large")]
    IntegerTooLarge,
    #[error("BER recursive parsing reached maximum depth")]
    BerMaxDepth,

    #[error("Invalid encoding or forbidden characters in string")]
    StringInvalidCharset,

    #[error("DER Failed constraint")]
    DerConstraintFailed,

    #[error("Feature is not yet implemented")]
    Unsupported,

    #[error("incomplete data, missing: {0:?}")]
    Incomplete(nom::Needed),

    #[error("nom error: {0:?}")]
    NomError(ErrorKind),
}

impl<'a> ParseError<&'a [u8]> for Error {
    fn from_error_kind(_input: &'a [u8], kind: ErrorKind) -> Self {
        Error::NomError(kind)
    }
    fn append(_input: &'a [u8], kind: ErrorKind, _other: Self) -> Self {
        Error::NomError(kind)
    }
}

impl From<Error> for nom::Err<Error> {
    fn from(e: Error) -> Self {
        nom::Err::Failure(e)
    }
}

impl From<str::Utf8Error> for Error {
    fn from(_: str::Utf8Error) -> Self {
        Error::StringInvalidCharset
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(_: string::FromUtf8Error) -> Self {
        Error::StringInvalidCharset
    }
}

pub type ParseResult<'a, T> = IResult<&'a [u8], T, Error>;

pub type Result<T> = std::result::Result<T, Error>;
