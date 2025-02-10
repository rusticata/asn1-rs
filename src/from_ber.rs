use core::convert::{TryFrom, TryInto};

use crate::{Any, Error, ParseResult};

/// Base trait for BER object parsers
///
/// Library authors should usually not directly implement this trait, but should prefer implementing the
/// [`TryFrom<Any>`] trait,
/// which offers greater flexibility and provides an equivalent `FromBer` implementation for free.
///
/// # Examples
///
/// ```
/// use asn1_rs::{Any, Result, Tag};
/// use std::convert::TryFrom;
///
/// // The type to be decoded
/// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// pub struct MyType(pub u32);
///
/// impl<'a> TryFrom<Any<'a>> for MyType {
///     type Error = asn1_rs::Error;
///
///     fn try_from(any: Any<'a>) -> Result<MyType> {
///         any.tag().assert_eq(Tag::Integer)?;
///         // for this fictive example, the type contains the number of characters
///         let n = any.data.len() as u32;
///         Ok(MyType(n))
///     }
/// }
///
/// // The above code provides a `FromBer` implementation for free.
///
/// // Example of parsing code:
/// use asn1_rs::FromBer;
///
/// let input = &[2, 1, 2];
/// // Objects can be parsed using `from_ber`, which returns the remaining bytes
/// // and the parsed object:
/// let (rem, my_type) = MyType::from_ber(input).expect("parsing failed");
/// ```
pub trait FromBer<'a, E = Error>: Sized {
    /// Attempt to parse input bytes into a BER object
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self, E>;
}

impl<'a, T, E> FromBer<'a, E> for T
where
    T: TryFrom<Any<'a>, Error = E>,
    E: From<Error>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, T, E> {
        let (i, any) = Any::from_ber(bytes).map_err(nom::Err::convert)?;
        let result = any.try_into().map_err(nom::Err::Error)?;
        Ok((i, result))
    }
}
