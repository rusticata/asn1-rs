#![allow(unknown_lints)]
#![allow(non_local_definitions)] // false positive for displaydoc::Display: https://github.com/yaahc/displaydoc/issues/46

use crate::{Class, Tag};
use alloc::str;
use alloc::string;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use core::fmt;
use displaydoc::Display;
use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::{IResult, Input};
#[cfg(feature = "std")]
use std::io;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Error)]
/// Error types for DER constraints
pub enum DerConstraint {
    /// Indefinite length not allowed
    IndefiniteLength,
    /// Object must not be constructed
    Constructed,
    /// Object must be constructed
    NotConstructed,
    /// DateTime object is missing timezone
    MissingTimeZone,
    /// DateTime object is missing seconds
    MissingSeconds,
    /// Bitstring unused bits must be set to zero
    UnusedBitsNotZero,
    /// Boolean value must be 0x00 of 0xff
    InvalidBoolean,
    /// Integer must not be empty
    IntegerEmpty,
    /// Leading zeroes in Integer encoding
    IntegerLeadingZeroes,
    /// Leading 0xff in negative Integer encoding
    IntegerLeadingFF,
}

/// The error type for operations of the [`FromBer`](crate::FromBer),
/// [`FromDer`](crate::FromDer), and associated traits.
#[derive(Clone, Debug, Display, PartialEq, Eq, Error)]
pub enum Error {
    /// BER object does not have the expected type
    BerTypeError,
    /// BER object does not have the expected value
    BerValueError,
    /// Invalid Length
    InvalidLength,
    /// Invalid Value when parsing object with tag {tag:?} {msg:}
    InvalidValue { tag: Tag, msg: String },
    /// Invalid Tag
    InvalidTag,
    /// Unknown tag: {0:?}
    UnknownTag(u32),
    /// Unexpected Tag (expected: {expected:?}, actual: {actual:?})
    UnexpectedTag { expected: Option<Tag>, actual: Tag },
    /// Unexpected Class (expected: {expected:?}, actual: {actual:?})
    UnexpectedClass {
        expected: Option<Class>,
        actual: Class,
    },

    /// Indefinite length not allowed
    IndefiniteLengthUnexpected,

    /// DER object was expected to be constructed (and found to be primitive)
    ConstructExpected,
    /// DER object was expected to be primitive (and found to be constructed)
    ConstructUnexpected,

    /// Integer too large to fit requested type
    IntegerTooLarge,
    /// BER integer is negative, while an unsigned integer was requested
    IntegerNegative,
    /// BER recursive parsing reached maximum depth
    BerMaxDepth,

    /// Invalid encoding or forbidden characters in string
    StringInvalidCharset,
    /// Invalid Date or Time
    InvalidDateTime,

    /// DER Failed constraint: {0:?}
    DerConstraintFailed(DerConstraint),

    /// Requesting borrowed data from a temporary object
    LifetimeError,
    /// Feature is not yet implemented
    Unsupported,

    /// incomplete data, missing: {0:?}
    Incomplete(nom::Needed),

    /// nom error: {0:?}
    NomError(ErrorKind),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub struct BerError<I: Input> {
    /// Input location where error happened
    input: I,
    /// Error kind
    inner_error: InnerError,
}

impl<I: Input> BerError<I> {
    /// Build a new error
    #[inline]
    pub const fn new(input: I, inner: InnerError) -> Self {
        Self {
            input,
            inner_error: inner,
        }
    }

    /// Return the error location
    pub const fn input(&self) -> &I {
        &self.input
    }

    /// Return the error kind
    pub fn inner(&self) -> &InnerError {
        &self.inner_error
    }

    #[inline]
    pub const fn nom_err(input: I, inner_error: InnerError) -> nom::Err<Self> {
        nom::Err::Error(Self { input, inner_error })
    }

    /// convert an `InnerError` to a `nom::Err::Error(input, inner)`
    ///
    /// This is usefull when mapping the result of a function returning `InnerError`
    /// inside a parser function (expecting error type `Err<Error<I>>`).
    ///
    /// For ex: `header.tag().assert_eq(Tag(0)).map_err(Error::convert(input))?`
    pub fn convert(input: I) -> impl Fn(InnerError) -> nom::Err<Self>
    where
        I: Clone,
    {
        move |inner_error| Self::nom_err(input.clone(), inner_error)
    }

    /// convert an `InnerError` to a `nom::Err::Error<E>` with `e: From<Error<I>>`
    ///
    /// This is similar to [`Self::convert`], but with an `Into` applied to the result
    pub fn convert_into<E: From<Self>>(input: I) -> impl Fn(InnerError) -> nom::Err<E>
    where
        I: Clone,
    {
        move |inner_error| nom::Err::Error(Self::new(input.clone(), inner_error).into())
    }

    /// Build an error from the provided invalid value
    #[inline]
    pub const fn invalid_value(input: I, tag: Tag, msg: String) -> Self {
        Self::new(input, InnerError::InvalidValue { tag, msg })
    }

    // /// Build an error from the provided unexpected class
    // #[inline]
    // pub const fn unexpected_class(expected: Option<Class>, actual: Class) -> Self {
    //     Self::UnexpectedClass { expected, actual }
    // }

    /// Build an error from the provided unexpected tag
    #[inline]
    pub const fn unexpected_tag(input: I, expected: Option<Tag>, actual: Tag) -> Self {
        Self::new(input, InnerError::UnexpectedTag { expected, actual })
    }

    /// Build an error from the provided unexpected tag
    #[inline]
    pub const fn unexpected_class(input: I, expected: Option<Class>, actual: Class) -> Self {
        Self::new(input, InnerError::UnexpectedClass { expected, actual })
    }

    /// Build an `Incomplete` error from the provided parameters
    #[inline]
    pub const fn incomplete(input: I, n: nom::Needed) -> Self {
        Self {
            input,
            inner_error: InnerError::Incomplete(n),
        }
    }
}

impl<'i> BerError<crate::Input<'i>> {
    /// Build a new `BerError` from input reference and inner error
    pub const fn err_input<'a: 'i, 'r>(input: &'r crate::Input<'a>, inner: InnerError) -> Self {
        BerError::new(input.const_clone(), inner)
    }

    /// Builda new `nom::Err::Error<BerError>` from input reference and inner error
    pub const fn nom_err_input<'a: 'i, 'r>(
        input: &'r crate::Input<'a>,
        inner: InnerError,
    ) -> nom::Err<Self> {
        nom::Err::Error(BerError::new(input.const_clone(), inner))
    }

    /// Consume the error and return error location and inner error
    pub fn consume(self) -> (crate::Input<'i>, InnerError) {
        (self.input, self.inner_error)
    }
}

impl<I: Input + fmt::Display> fmt::Display for BerError<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BerError: input={} error={}", self.input(), self.inner())
    }
}

impl<I: Input> ParseError<I> for BerError<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        Self {
            input,
            inner_error: InnerError::Nom(kind),
        }
    }

    fn append(_: I, _z: ErrorKind, other: Self) -> Self {
        // NOTE: we do not support error stacking, and just use the last one
        other
    }
}

impl From<Error> for BerError<crate::Input<'_>> {
    fn from(value: Error) -> Self {
        let inner = InnerError::from(value);
        BerError::new(crate::Input::from_slice(&[]), inner)
    }
}

impl<'a, 'b> From<(crate::Input<'b>, Error)> for BerError<crate::Input<'a>>
where
    'b: 'a,
{
    fn from(value: (crate::Input<'b>, Error)) -> Self {
        Self {
            input: value.0,
            inner_error: InnerError::from(value.1),
        }
    }
}

impl<'a, 'b> From<(crate::Input<'b>, BerError<crate::Input<'b>>)> for BerError<crate::Input<'a>>
where
    'b: 'a,
{
    fn from(value: (crate::Input<'b>, BerError<crate::Input<'b>>)) -> Self {
        // ignore new input, assume error input is already set
        value.1
        // Self {
        //     input: value.0,
        //     inner_error: value.1.inner_error,
        // }
    }
}

/// The error type for operations of the [`FromBer`](crate::FromBer),
/// [`FromDer`](crate::FromDer), and associated traits.
#[derive(Clone, Debug, Display, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum InnerError {
    /// BER object does not have the expected type
    BerTypeError,
    /// BER object does not have the expected value
    BerValueError,
    /// Invalid Length
    InvalidLength,
    /// Invalid Value when parsing object with tag {tag:?} {msg:}
    InvalidValue { tag: Tag, msg: String },
    /// Invalid Tag
    InvalidTag,
    /// Unknown tag: {0:?}
    UnknownTag(u32),
    /// Unexpected Tag (expected: {expected:?}, actual: {actual:?})
    UnexpectedTag { expected: Option<Tag>, actual: Tag },
    /// Unexpected Class (expected: {expected:?}, actual: {actual:?})
    UnexpectedClass {
        expected: Option<Class>,
        actual: Class,
    },

    /// Indefinite length not allowed
    IndefiniteLengthUnexpected,

    /// DER object was expected to be constructed (and found to be primitive)
    ConstructExpected,
    /// DER object was expected to be primitive (and found to be constructed)
    ConstructUnexpected,

    /// Integer too large to fit requested type
    IntegerTooLarge,
    /// BER integer is negative, while an unsigned integer was requested
    IntegerNegative,
    /// BER recursive parsing reached maximum depth
    BerMaxDepth,

    /// Invalid encoding or forbidden characters in string
    StringInvalidCharset,
    /// Invalid Date or Time
    InvalidDateTime,

    /// DER Failed constraint: {0:?}
    DerConstraintFailed(DerConstraint),

    /// Parse error
    Nom(ErrorKind),

    /// Requesting borrowed data from a temporary object
    LifetimeError,
    /// Feature is not yet implemented
    Unsupported,

    /// incomplete data, missing: {0:?}
    Incomplete(nom::Needed),
}

impl InnerError {
    pub fn invalid_value(tag: Tag, msg: &str) -> Self {
        Self::InvalidValue {
            tag,
            msg: msg.to_string(),
        }
    }
}

impl From<str::Utf8Error> for InnerError {
    fn from(_: str::Utf8Error) -> Self {
        InnerError::StringInvalidCharset
    }
}

impl From<string::FromUtf8Error> for InnerError {
    fn from(_: string::FromUtf8Error) -> Self {
        InnerError::StringInvalidCharset
    }
}

impl From<string::FromUtf16Error> for InnerError {
    fn from(_: string::FromUtf16Error) -> Self {
        InnerError::StringInvalidCharset
    }
}

impl From<Error> for InnerError {
    fn from(value: Error) -> Self {
        match value {
            Error::BerTypeError => Self::BerTypeError,
            Error::BerValueError => Self::BerValueError,
            Error::InvalidLength => Self::InvalidLength,
            Error::InvalidValue { tag, msg } => Self::InvalidValue { tag, msg },
            Error::InvalidTag => Self::InvalidTag,
            Error::UnknownTag(tag) => Self::UnknownTag(tag),
            Error::UnexpectedTag { expected, actual } => Self::UnexpectedTag { expected, actual },
            Error::UnexpectedClass { expected, actual } => {
                Self::UnexpectedClass { expected, actual }
            }
            Error::IndefiniteLengthUnexpected => Self::IndefiniteLengthUnexpected,
            Error::ConstructExpected => Self::ConstructExpected,
            Error::ConstructUnexpected => Self::ConstructUnexpected,
            Error::IntegerTooLarge => Self::IntegerTooLarge,
            Error::IntegerNegative => Self::IntegerNegative,
            Error::BerMaxDepth => Self::BerMaxDepth,
            Error::StringInvalidCharset => Self::StringInvalidCharset,
            Error::InvalidDateTime => Self::InvalidDateTime,
            Error::DerConstraintFailed(der_constraint) => Self::DerConstraintFailed(der_constraint),
            Error::LifetimeError => Self::LifetimeError,
            Error::Unsupported => Self::Unsupported,
            Error::Incomplete(needed) => Self::Incomplete(needed),
            Error::NomError(error_kind) => Self::Nom(error_kind),
        }
    }
}

impl Error {
    /// Build an error from the provided invalid value
    #[inline]
    pub const fn invalid_value(tag: Tag, msg: String) -> Self {
        Self::InvalidValue { tag, msg }
    }

    /// Build an error from the provided unexpected class
    #[inline]
    pub const fn unexpected_class(expected: Option<Class>, actual: Class) -> Self {
        Self::UnexpectedClass { expected, actual }
    }

    /// Build an error from the provided unexpected tag
    #[inline]
    pub const fn unexpected_tag(expected: Option<Tag>, actual: Tag) -> Self {
        Self::UnexpectedTag { expected, actual }
    }

    /// Build an error from a `Nom::Err<BerError<Input>>`
    pub fn from_nom_berr(e: nom::Err<BerError<crate::Input<'_>>>) -> Self {
        match e {
            nom::Err::Incomplete(n) => Self::Incomplete(n),
            nom::Err::Error(e) | nom::Err::Failure(e) => e.inner_error.into(),
        }
    }
}

impl<I> ParseError<I> for Error {
    fn from_error_kind(_input: I, kind: ErrorKind) -> Self {
        Error::NomError(kind)
    }
    fn append(_input: I, kind: ErrorKind, _other: Self) -> Self {
        Error::NomError(kind)
    }
}

impl From<Error> for nom::Err<Error> {
    fn from(e: Error) -> Self {
        nom::Err::Error(e)
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

impl From<string::FromUtf16Error> for Error {
    fn from(_: string::FromUtf16Error) -> Self {
        Error::StringInvalidCharset
    }
}

impl From<nom::Err<Error>> for Error {
    fn from(e: nom::Err<Error>) -> Self {
        match e {
            nom::Err::Incomplete(n) => Self::Incomplete(n),
            nom::Err::Error(e) | nom::Err::Failure(e) => e,
        }
    }
}

impl<I: Input> From<BerError<I>> for Error {
    fn from(value: BerError<I>) -> Self {
        Self::from(value.inner_error)
    }
}

impl From<InnerError> for Error {
    fn from(value: InnerError) -> Self {
        match value {
            InnerError::BerTypeError => Self::BerTypeError,
            InnerError::BerValueError => Self::BerValueError,
            InnerError::InvalidLength => Self::InvalidLength,
            InnerError::InvalidValue { tag, msg } => Self::InvalidValue { tag, msg },
            InnerError::InvalidTag => Self::InvalidTag,
            InnerError::UnknownTag(t) => Self::UnknownTag(t),
            InnerError::UnexpectedTag { expected, actual } => {
                Self::UnexpectedTag { expected, actual }
            }
            InnerError::UnexpectedClass { expected, actual } => {
                Self::unexpected_class(expected, actual)
            }
            InnerError::IndefiniteLengthUnexpected => Self::IndefiniteLengthUnexpected,
            InnerError::ConstructExpected => Self::ConstructExpected,
            InnerError::ConstructUnexpected => Self::ConstructUnexpected,
            InnerError::IntegerTooLarge => Self::IntegerTooLarge,
            InnerError::IntegerNegative => Self::IntegerNegative,
            InnerError::BerMaxDepth => Self::BerMaxDepth,
            InnerError::StringInvalidCharset => Self::StringInvalidCharset,
            InnerError::InvalidDateTime => Self::InvalidDateTime,
            InnerError::DerConstraintFailed(der_constraint) => {
                Self::DerConstraintFailed(der_constraint)
            }
            InnerError::Nom(error_kind) => Self::NomError(error_kind),
            InnerError::LifetimeError => Self::LifetimeError,
            InnerError::Unsupported => Self::Unsupported,
            InnerError::Incomplete(n) => Self::Incomplete(n),
        }
    }
}

impl<I, E> FromExternalError<I, E> for Error {
    fn from_external_error(_input: I, kind: ErrorKind, _e: E) -> Error {
        Error::NomError(kind)
    }
}

/// Flatten all `nom::Err` variants error into a single error type
pub fn from_nom_error<E, F>(e: nom::Err<E>) -> F
where
    F: From<E> + From<Error>,
{
    match e {
        nom::Err::Error(e) | nom::Err::Failure(e) => F::from(e),
        nom::Err::Incomplete(n) => F::from(Error::Incomplete(n)),
    }
}

/// Flatten all `nom::Err` variants error into a single error type
///
/// Note: used in custom derive
pub fn from_nom_bererror<'a, E, F>(input: crate::Input<'a>, e: nom::Err<E>) -> F
where
    F: Sized + From<(crate::Input<'a>, E)> + From<Error>,
{
    match e {
        nom::Err::Error(e) | nom::Err::Failure(e) => F::from((input, e)),
        nom::Err::Incomplete(n) => F::from(Error::Incomplete(n)),
    }
}

/// Holds the result of BER/DER serialization functions
pub type ParseResult<'a, T, E = Error> = IResult<&'a [u8], T, E>;

/// A specialized `Result` type for all operations from this crate.
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// The error type for serialization operations of the [`ToDer`](crate::ToDer) trait.
#[cfg(feature = "std")]
#[derive(Debug, Error)]
pub enum SerializeError {
    #[error("ASN.1 error: {0:?}")]
    ASN1Error(#[from] Error),

    #[error("Invalid Class {class:}")]
    InvalidClass { class: u8 },

    #[error("Invalid Length")]
    InvalidLength,

    #[error("I/O error: {0:?}")]
    IOError(#[from] io::Error),
}

#[cfg(feature = "std")]
/// Holds the result of BER/DER encoding functions
pub type SerializeResult<T> = std::result::Result<T, SerializeError>;
