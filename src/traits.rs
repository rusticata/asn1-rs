use crate::error::*;
use crate::{Any, Tag};
use std::convert::{TryFrom, TryInto};
use std::io::Write;

/// Phantom type representing a BER parser
#[doc(hidden)]
#[derive(Debug)]
pub enum BerParser {}

/// Phantom type representing a DER parser
#[doc(hidden)]
#[derive(Debug)]
pub enum DerParser {}

#[doc(hidden)]
pub trait ASN1Parser {}

impl ASN1Parser for BerParser {}
impl ASN1Parser for DerParser {}

pub trait Tagged {
    const TAG: Tag;
}

pub trait FromBer<'a>: Sized {
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self>;
}

impl<'a, T> FromBer<'a> for T
where
    T: TryFrom<Any<'a>, Error = Error>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<T> {
        let (i, any) = Any::from_ber(bytes)?;
        let result = any.try_into().map_err(nom::Err::Failure)?;
        Ok((i, result))
    }
}

pub trait FromDer<'a>: Sized {
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self>;
}

impl<'a, T> FromDer<'a> for T
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: CheckDerConstraints,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<T> {
        let (i, any) = Any::from_der(bytes)?;
        // X.690 section 10.1: definite form of length encoding shall be used
        if !any.header.length.is_definite() {
            return Err(nom::Err::Failure(Error::IndefiniteLengthUnexpected));
        }
        <T as CheckDerConstraints>::check_constraints(&any).map_err(nom::Err::Failure)?;
        let result = any.try_into().map_err(nom::Err::Failure)?;
        Ok((i, result))
    }
}

pub trait CheckDerConstraints {
    fn check_constraints(any: &Any) -> Result<()>;
}

pub trait ToDer {
    fn to_vec(&self) -> Vec<u8>;

    // XXX to be adjusted
    fn to_der(&self, writer: &mut dyn Write) -> ParseResult<usize>;
}

pub trait ToStatic {
    type Owned: 'static;
    fn to_static(&self) -> Self::Owned;
}
