use core::fmt::Debug;

use crate::{error::*, BerParser};
use crate::{Class, Explicit, Implicit, TaggedParser};

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

/// Helper trait for creating tagged EXPLICIT values
///
/// # Examples
///
/// ```
/// use asn1_rs::{AsTaggedExplicit, Class, TaggedParser};
///
/// // create a `[1] EXPLICIT INTEGER` value
/// let tagged: TaggedParser<_, _, _> = 4u32.explicit(Class::ContextSpecific, 1);
/// ```
pub trait AsTaggedExplicit<'a, E = Error>: Sized {
    fn explicit(self, class: Class, tag: u32) -> TaggedParser<'a, Explicit, Self, E> {
        TaggedParser::new_explicit(class, tag, self)
    }
}

impl<'a, T, E> AsTaggedExplicit<'a, E> for T where T: BerParser<'a, Error = E> {}

/// Helper trait for creating tagged IMPLICIT values
///
/// # Examples
///
/// ```
/// use asn1_rs::{AsTaggedImplicit, Class, TaggedParser};
///
/// // create a `[1] IMPLICIT INTEGER` value, not constructed
/// let tagged: TaggedParser<_, _, _> = 4u32.implicit(Class::ContextSpecific, false, 1);
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

impl<'a, T, E> AsTaggedImplicit<'a, E> for T where T: BerParser<'a, Error = E> {}

pub use crate::tostatic::*;
