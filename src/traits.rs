use crate::error::*;
#[cfg(feature = "std")]
use crate::DynTagged;
use crate::{Class, Explicit, Implicit, TaggedParser};
use core::fmt::Debug;
#[cfg(feature = "std")]
use std::io::Write;

/// Phantom type representing a BER parser
// TODO: rename this to `BerMode`?
#[doc(hidden)]
#[derive(Debug)]
pub enum BerMode {}

/// Phantom type representing a DER parser
#[doc(hidden)]
#[derive(Debug)]
pub enum DerMode {}

#[doc(hidden)]
pub trait ASN1Mode {}

impl ASN1Mode for BerMode {}
impl ASN1Mode for DerMode {}

/// Common trait for all objects that can be encoded using the DER representation
///
/// # Examples
///
/// Objects from this crate can be encoded as DER:
///
/// ```
/// use asn1_rs::{Integer, ToDer};
///
/// let int = Integer::from(4u32);
/// let mut writer = Vec::new();
/// let sz = int.write_der(&mut writer).expect("serialization failed");
///
/// assert_eq!(&writer, &[0x02, 0x01, 0x04]);
/// # assert_eq!(sz, 3);
/// ```
///
/// Many of the primitive types can also directly be encoded as DER:
///
/// ```
/// use asn1_rs::ToDer;
///
/// let mut writer = Vec::new();
/// let sz = 4.write_der(&mut writer).expect("serialization failed");
///
/// assert_eq!(&writer, &[0x02, 0x01, 0x04]);
/// # assert_eq!(sz, 3);
/// ```
#[cfg(feature = "std")]
pub trait ToDer
where
    Self: DynTagged,
{
    /// Get the length of the object (including the header), when encoded
    ///
    // Since we are using DER, length cannot be Indefinite, so we can use `usize`.
    // XXX can this function fail?
    fn to_der_len(&self) -> Result<usize>;

    /// Write the DER encoded representation to a newly allocated `Vec<u8>`.
    fn to_der_vec(&self) -> SerializeResult<Vec<u8>> {
        let mut v = Vec::new();
        let _ = self.write_der(&mut v)?;
        Ok(v)
    }

    /// Similar to using `to_vec`, but uses provided values without changes.
    /// This can generate an invalid encoding for a DER object.
    fn to_der_vec_raw(&self) -> SerializeResult<Vec<u8>> {
        let mut v = Vec::new();
        let _ = self.write_der_raw(&mut v)?;
        Ok(v)
    }

    /// Attempt to write the DER encoded representation (header and content) into this writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use asn1_rs::{Integer, ToDer};
    ///
    /// let int = Integer::from(4u32);
    /// let mut writer = Vec::new();
    /// let sz = int.write_der(&mut writer).expect("serialization failed");
    ///
    /// assert_eq!(&writer, &[0x02, 0x01, 0x04]);
    /// # assert_eq!(sz, 3);
    /// ```
    fn write_der(&self, writer: &mut dyn Write) -> SerializeResult<usize> {
        let sz = self.write_der_header(writer)?;
        let sz = sz + self.write_der_content(writer)?;
        Ok(sz)
    }

    /// Attempt to write the DER header to this writer.
    fn write_der_header(&self, writer: &mut dyn Write) -> SerializeResult<usize>;

    /// Attempt to write the DER content (all except header) to this writer.
    fn write_der_content(&self, writer: &mut dyn Write) -> SerializeResult<usize>;

    /// Similar to using `to_der`, but uses provided values without changes.
    /// This can generate an invalid encoding for a DER object.
    fn write_der_raw(&self, writer: &mut dyn Write) -> SerializeResult<usize> {
        self.write_der(writer)
    }
}

#[cfg(feature = "std")]
impl<'a, T> ToDer for &'a T
where
    T: ToDer,
    &'a T: DynTagged,
{
    fn to_der_len(&self) -> Result<usize> {
        (*self).to_der_len()
    }

    fn write_der_header(&self, writer: &mut dyn Write) -> SerializeResult<usize> {
        (*self).write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn Write) -> SerializeResult<usize> {
        (*self).write_der_content(writer)
    }
}

/// Helper trait for creating tagged EXPLICIT values
///
/// # Examples
///
/// ```
/// use asn1_rs::{AsTaggedExplicit, Class, Error, TaggedParser};
///
/// // create a `[1] EXPLICIT INTEGER` value
/// let tagged: TaggedParser<_, _, Error> = 4u32.explicit(Class::ContextSpecific, 1);
/// ```
pub trait AsTaggedExplicit<'a, E = Error>: Sized {
    fn explicit(self, class: Class, tag: u32) -> TaggedParser<'a, Explicit, Self, E> {
        TaggedParser::new_explicit(class, tag, self)
    }
}

impl<'a, T, E> AsTaggedExplicit<'a, E> for T where T: Sized + 'a {}

/// Helper trait for creating tagged IMPLICIT values
///
/// # Examples
///
/// ```
/// use asn1_rs::{AsTaggedImplicit, Class, Error, TaggedParser};
///
/// // create a `[1] IMPLICIT INTEGER` value, not constructed
/// let tagged: TaggedParser<_, _, Error> = 4u32.implicit(Class::ContextSpecific, false, 1);
/// ```
pub trait AsTaggedImplicit<'a, E = Error>: Sized {
    fn implicit(
        self,
        class: Class,
        constructed: bool,
        tag: u32,
    ) -> TaggedParser<'a, Implicit, Self, E> {
        TaggedParser::new_implicit(class, constructed, tag, self)
    }
}

impl<'a, T, E> AsTaggedImplicit<'a, E> for T where T: Sized + 'a {}

pub use crate::tostatic::*;
