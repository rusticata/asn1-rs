use core::convert::{TryFrom, TryInto};
use core::fmt::{Debug, Display};

use crate::debug::{trace, trace_generic};
use crate::{parse_der_any, Any, Error, ParseResult, Result};

/// Base trait for DER object parsers
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
                let (i, any) = trace(core::any::type_name::<T>(), parse_der_any, bytes)
                    .map_err(nom::Err::convert)?;
                <T as CheckDerConstraints>::check_constraints(&any)
                    .map_err(|e| nom::Err::Error(e.into()))?;
                let result = any.try_into().map_err(nom::Err::Error)?;
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
