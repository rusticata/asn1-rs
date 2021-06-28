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

/// Base trait for BER object parsers
///
/// Library authors should usually not directly implement this trait, but should prefer implementing the
/// `TryFrom<Any>` trait,
/// which offers greater flexibility and provides an equivalent `BerParser` implementation for free.
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

/// Base trait for DER object parsers
///
/// Library authors should usually not directly implement this trait, but should prefer implementing the
/// `TryFrom<Any>` + `CheckDerConstraint` traits,
/// which offers greater flexibility and provides an equivalent `DerParser` implementation for free.

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

/// Verification of DER constraints
pub trait CheckDerConstraints {
    fn check_constraints(any: &Any) -> Result<()>;
}

pub trait ToDer {
    /// Get the length of the object, when encoded
    ///
    // Since we are using DER, length cannot be Indefinite, so we can use `usize`.
    // XXX can this function fail?
    fn to_der_len(&self) -> Result<usize>;

    /// Write the DER encoded representation to a newly allocated `Vec<u8>`.
    fn to_der_vec(&self) -> SerializeResult<Vec<u8>> {
        let mut v = Vec::new();
        let _ = self.to_der(&mut v)?;
        Ok(v)
    }

    // Write the DER encoded representation to `writer`.
    fn to_der(&self, writer: &mut dyn Write) -> SerializeResult<usize>;

    /// Similar to using `to_der`, but uses provided values without changes.
    /// This can generate an invalid encoding for a DER object.
    fn to_der_raw(&self, writer: &mut dyn Write) -> SerializeResult<usize> {
        self.to_der(writer)
    }

    /// Similar to using `to_vec`, but uses provided values without changes.
    /// This can generate an invalid encoding for a DER object.
    fn to_der_vec_raw(&self) -> SerializeResult<Vec<u8>> {
        let mut v = Vec::new();
        let _ = self.to_der_raw(&mut v)?;
        Ok(v)
    }
}

impl<T> ToDer for &'_ T
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        (*self).to_der_len()
    }

    fn to_der(&self, writer: &mut dyn Write) -> SerializeResult<usize> {
        (*self).to_der(writer)
    }
}

pub trait ToStatic {
    type Owned: 'static;
    fn to_static(&self) -> Self::Owned;
}
