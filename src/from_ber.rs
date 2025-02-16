use core::convert::{TryFrom, TryInto};

use nom::bytes::streaming::take;
use nom::error::ParseError;
use nom::{Err, IResult, Input};

use crate::{Any, BerError, Error, Header, ParseResult, Tag};

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
        let (i, any) = Any::from_ber(bytes).map_err(Err::convert)?;
        let result = any.try_into().map_err(Err::Error)?;
        Ok((i, result))
    }
}

pub trait BerParser<'a, I: Input<Item = u8> + 'a>: Sized
where
    I: 'a,
{
    type Error: ParseError<I> + From<BerError<I>>;

    /// Attempt to parse a new BER object from data.
    ///
    /// Header tag must match expected tag
    fn parse_ber(input: I) -> IResult<I, Self, Self::Error> {
        let (rem, header) = Header::parse_ber(input.clone()).map_err(Err::convert)?;
        // TODO: handle indefinite
        let length = header
            .length
            .definite_inner()
            .map_err(BerError::convert_into(input.clone()))?;
        if !Self::check_tag(header.tag) {
            return Err(Err::Error(
                // TODO: expected Tag is `None`, so the error will not be helpful
                BerError::unexpected_tag(input, None, header.tag).into(),
            ));
        }
        let (rem, data) = take(length)(rem)?;
        let (_, obj) = Self::from_any_ber(data, header).map_err(Err::convert)?;
        Ok((rem, obj))
    }

    /// Check if provided tag is acceptable
    ///
    /// Return `true` if tag can match current object.
    fn check_tag(_tag: Tag) -> bool {
        true
    }

    /// Parse a new BER object from header and data.
    ///
    /// `input` length is guaranteed to match `header` length (definite or indefinite)
    ///
    /// Note: in this method, implementers should *not* check header tag (which can be
    /// different from the usual object tag when using IMPLICIT tagging, for ex.).
    // TODO: when header is generic, remove this lifetime and use <I>
    fn from_any_ber(input: I, header: Header<'a>) -> IResult<I, Self, Self::Error>;

    fn parse_ber_optional(input: I) -> IResult<I, Option<Self>, Self::Error> {
        if input.input_len() == 0 {
            return Ok((input, None));
        }
        let (rem, header) = Header::parse_ber(input.clone()).map_err(Err::convert)?;
        if !Self::check_tag(header.tag) {
            return Ok((input, None));
        }
        // TODO: handle indefinite
        let length = header
            .length
            .definite_inner()
            .map_err(BerError::convert_into(input.clone()))?;
        let (rem, data) = take(length)(rem)?;
        let (_, obj) = Self::from_any_ber(data, header).map_err(Err::convert)?;
        Ok((rem, Some(obj)))
    }
}
