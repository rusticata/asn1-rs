use core::convert::{TryFrom, TryInto};
use core::fmt::{Debug, Display};

use nom::bytes::streaming::take;
use nom::error::ParseError;
use nom::{Err, IResult, Input as _};

use crate::debug::{trace, trace_generic};
use crate::{
    parse_der_any, wrap_ber_parser, Any, BerError, DynTagged, Error, Header, Input, ParseResult,
    Result,
};

/// Base trait for DER object parsers
///
/// # Notes
///
/// *This trait might become deprecated soon! Instead of this one, implement trait [`DerParser`].*
///
/// Library authors should usually not directly implement this trait, but should prefer implementing the
/// [`TryFrom<Any>`] + [`CheckDerConstraints`] traits,
/// which offers greater flexibility and provides an equivalent `FromDer` implementation for free
/// (in fact, it provides both [`FromBer`](crate::FromBer) and [`FromDer`]).
///
/// Note: if you already implemented [`TryFrom<Any>`] and [`CheckDerConstraints`],
/// you can get a free [`FromDer`] implementation by implementing the
/// [`DerAutoDerive`] trait. This is not automatic, so it is also possible to manually
/// implement [`FromDer`] if preferred.
///
/// # Examples
///
/// ```
/// use asn1_rs::{Any, CheckDerConstraints, DerAutoDerive, Result, Tag};
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
/// impl CheckDerConstraints for MyType {
///     fn check_constraints(any: &Any) -> Result<()> {
///         any.header.assert_primitive()?;
///         Ok(())
///     }
/// }
///
/// impl DerAutoDerive for MyType {}
///
/// // The above code provides a `FromDer` implementation for free.
///
/// // Example of parsing code:
/// use asn1_rs::FromDer;
///
/// let input = &[2, 1, 2];
/// // Objects can be parsed using `from_der`, which returns the remaining bytes
/// // and the parsed object:
/// let (rem, my_type) = MyType::from_der(input).expect("parsing failed");
/// ```
pub trait FromDer<'a, E = Error>: Sized {
    /// Attempt to parse input bytes into a DER object (enforcing constraints)
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E>;
}

/// Trait to automatically derive `FromDer`
///
/// This trait is only a marker to control if a DER parser should be automatically derived. It is
/// empty.
///
/// This trait is used in combination with others:
/// after implementing [`TryFrom<Any>`] and [`CheckDerConstraints`] for a type,
/// a free [`FromDer`] implementation is provided by implementing the
/// [`DerAutoDerive`] trait. This is the most common case.
///
/// However, this is not automatic so it is also possible to manually
/// implement [`FromDer`] if preferred.
/// Manual implementation is generally only needed for generic containers (for ex. `Vec<T>`),
/// because the default implementation adds a constraint on `T` to implement also `TryFrom<Any>`
/// and `CheckDerConstraints`. This is problematic when `T` only provides `FromDer`, and can be
/// solved by providing a manual implementation of [`FromDer`].
pub trait DerAutoDerive {}

impl<'a, T, E> FromDer<'a, E> for T
where
    T: TryFrom<Any<'a>, Error = E>,
    T: CheckDerConstraints,
    T: DerAutoDerive,
    E: From<Error> + Display + Debug,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, T, E> {
        trace_generic(
            core::any::type_name::<T>(),
            "T::from_der",
            |bytes| {
                let (i, any) = trace(
                    core::any::type_name::<T>(),
                    wrap_ber_parser(parse_der_any),
                    bytes,
                )
                .map_err(Err::convert)?;
                <T as CheckDerConstraints>::check_constraints(&any)
                    .map_err(|e| Err::Error(e.into()))?;
                let result = any.try_into().map_err(Err::Error)?;
                Ok((i, result))
            },
            bytes,
        )
    }
}

/// Verification of DER constraints
pub trait CheckDerConstraints {
    fn check_constraints(any: &Any) -> Result<()>;
}

/// Base trait for DER object parsers
///
/// Implementers should provide a definition for the following:
/// - method [`from_der_content`](DerParser::from_der_content): Parse DER content, given a header and data
/// - trait [`DynTagged`]
///
/// This trait can be automatically derived from a `struct` using the [`DerParserSequence`](crate::DerParserSequence)
/// or [`DerParserSet`](crate::DerParserSet) custom derive attributes.
pub trait DerParser<'i>
where
    Self: Sized,
    Self: DynTagged,
{
    /// The Error type for parsing errors.
    type Error: ParseError<Input<'i>> + From<BerError<Input<'i>>>;

    /// Attempt to parse a new DER object from data.
    ///
    /// Header tag must match expected tag, and length must be definite.
    fn parse_der(input: Input<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, header) = Header::parse_der(input.clone()).map_err(Err::convert)?;
        // get length, rejecting indefinite (invalid for DER)
        let length = header
            .length
            .definite_inner()
            .map_err(BerError::convert_into(input.clone()))?;
        if !Self::accept_tag(header.tag) {
            return Err(Err::Error(
                // TODO: expected Tag is `None`, so the error will not be helpful
                BerError::unexpected_tag(input, None, header.tag).into(),
            ));
        }
        let (rem, data) = take(length)(rem)?;
        let (_, obj) = Self::from_der_content(&header, data).map_err(Err::convert)?;
        Ok((rem, obj))
    }

    /// Parse a new DER object from header and data.
    ///
    /// `input` length is (supposed to be) guaranteed to match `header` length (definite)
    ///
    /// This function also checks DER-related constraints.
    /// Relevant sections in specifications:
    /// - Canonical encoding rules (X.690: 9)
    /// - Distinguished encoding rules (X.690: 10)
    /// - Restrictions on BER employed by both CER and DER (X.690: 11)
    ///
    /// Note: in this method, implementers should *not* check header tag (which can be
    /// different from the usual object tag when using IMPLICIT tagging, for ex.).
    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error>;

    fn parse_der_optional(input: Input<'i>) -> IResult<Input<'i>, Option<Self>, Self::Error> {
        if input.input_len() == 0 {
            return Ok((input, None));
        }
        let (rem, header) = Header::parse_der(input.clone()).map_err(Err::convert)?;
        if !Self::accept_tag(header.tag) {
            return Ok((input, None));
        }
        // get length, rejecting indefinite (invalid for DER)
        let length = header
            .length
            .definite_inner()
            .map_err(BerError::convert_into(input.clone()))?;
        let (rem, data) = take(length)(rem)?;
        let (_, obj) = Self::from_der_content(&header, data).map_err(Err::convert)?;
        Ok((rem, Some(obj)))
    }
}
