use core::convert::{TryFrom, TryInto};
use core::fmt::Display;

use nom::error::ParseError;
use nom::Input as _;
use nom::{Err, IResult};

use crate::ber::{GetObjectContent, MAX_RECURSION};
use crate::debug::macros::log_error;
use crate::debug::trace_input;
use crate::{Any, BerError, BerMode, DynTagged, Error, Header, Input, ParseResult};

/// Base trait for BER object parsers
///
/// # Notes
///
/// *This trait might become deprecated soon! Instead of this one, implement trait [`BerParser`].*
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
        match any.try_into().map_err(Err::Error) {
            Ok(result) => Ok((i, result)),
            Err(err) => {
                log_error!(
                    "≠ Conversion from Any to {} failed",
                    core::any::type_name::<T>()
                );
                Err(err)
            }
        }
    }
}

/// Base trait for BER object parsers
///
/// Implementers should provide a definition for the following:
/// - method [`from_ber_content`](BerParser::from_ber_content): Parse BER content, given a header and data
/// - trait [`DynTagged`]
///
/// This trait can be automatically derived from a `struct` using the [`BerParserSequence`](crate::BerParserSequence)
/// or [`BerParserSet`](crate::BerParserSet) custom derive attributes.
pub trait BerParser<'i>
where
    Self: Sized,
    Self: DynTagged,
{
    /// The Error type for parsing errors.
    type Error: Display + ParseError<Input<'i>> + From<BerError<Input<'i>>>;

    /// Attempt to parse a new BER object from data.
    ///
    /// Header tag must match expected tag
    fn parse_ber(input: Input<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        trace_input("DerParser::parse_ber", |input| {
            let (rem, header) = Header::parse_ber(input.clone()).map_err(Err::convert)?;
            if !Self::accept_tag(header.tag) {
                return Err(Err::Error(
                    // FIXME: expected Tag is `None`, so the error will not be helpful
                    BerError::unexpected_tag(input, None, header.tag).into(),
                ));
            }
            let (rem, data) =
                BerMode::get_object_content(&header, rem, MAX_RECURSION).map_err(Err::convert)?;
            let (_, obj) = trace_input("BerParser::from_ber_content", |i| {
                // wrap from_ber_content function to display better errors, if any
                Self::from_ber_content(&header, i)
            })(data)
            .map_err(Err::convert)?;
            Ok((rem, obj))
        })(input)
    }

    /// Parse a new BER object from header and data.
    ///
    /// `input` length is guaranteed to match `header` length (definite or indefinite)
    ///
    /// Note: in this method, implementers should *not* check header tag (which can be
    /// different from the usual object tag when using IMPLICIT tagging, for ex.).
    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error>;

    fn parse_ber_optional(input: Input<'i>) -> IResult<Input<'i>, Option<Self>, Self::Error> {
        if input.input_len() == 0 {
            return Ok((input, None));
        }
        let (rem, header) = Header::parse_ber(input.clone()).map_err(Err::convert)?;
        if !Self::accept_tag(header.tag) {
            return Ok((input, None));
        }
        let (rem, data) =
            BerMode::get_object_content(&header, rem, MAX_RECURSION).map_err(Err::convert)?;
        let (_, obj) = Self::from_ber_content(&header, data).map_err(Err::convert)?;
        Ok((rem, Some(obj)))
    }
}

// NOTE: function useful during transition to Input. Remove this after
pub(crate) fn wrap_ber_parser<'i, F, T>(mut f: F) -> impl FnMut(&'i [u8]) -> ParseResult<'i, T>
where
    F: FnMut(Input<'i>) -> IResult<Input<'i>, T, BerError<Input<'i>>>,
{
    move |i: &[u8]| {
        let input = Input::from_slice(i);
        match f(input) {
            Ok((rem, res)) => Ok((rem.into_bytes(), res)),
            Err(e) => Err(Err::convert(e)),
        }
    }
}
