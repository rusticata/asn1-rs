use crate::*;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::{Debug, Display};
use core::iter::FromIterator;
use core::ops::{Deref, DerefMut};

use self::debug::{trace, trace_generic};

/// The `SEQUENCE OF` object is an ordered list of homogeneous types.
///
/// This type implements `Deref<Target = [T]>` and `DerefMut<Target = [T]>`, so all methods
/// like `.iter()`, `.len()` and others can be used transparently as if using a vector.
///
/// # Examples
///
/// ```
/// use asn1_rs::SequenceOf;
/// use std::iter::FromIterator;
///
/// // build set
/// let seq = SequenceOf::from_iter([2, 3, 4]);
///
/// // `seq` now contains the serialized DER representation of the array
///
/// // iterate objects
/// let mut sum = 0;
/// for item in seq.iter() {
///     // item has type `Result<u32>`, since parsing the serialized bytes could fail
///     sum += *item;
/// }
/// assert_eq!(sum, 9);
///
/// ```
#[derive(Debug, PartialEq)]
pub struct SequenceOf<T> {
    pub(crate) items: Vec<T>,
}

impl<T> SequenceOf<T> {
    /// Builds a `SEQUENCE OF` from the provided content
    #[inline]
    pub const fn new(items: Vec<T>) -> Self {
        SequenceOf { items }
    }

    /// Converts `self` into a vector without clones or allocation.
    #[inline]
    pub fn into_vec(self) -> Vec<T> {
        self.items
    }

    /// Appends an element to the back of a collection
    #[inline]
    pub fn push(&mut self, item: T) {
        self.items.push(item)
    }
}

impl<T> AsRef<[T]> for SequenceOf<T> {
    fn as_ref(&self) -> &[T] {
        &self.items
    }
}

impl<T> AsMut<[T]> for SequenceOf<T> {
    fn as_mut(&mut self) -> &mut [T] {
        &mut self.items
    }
}

impl<T> Deref for SequenceOf<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DerefMut for SequenceOf<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<T> From<SequenceOf<T>> for Vec<T> {
    fn from(seq: SequenceOf<T>) -> Self {
        seq.items
    }
}

impl<T> FromIterator<T> for SequenceOf<T> {
    fn from_iter<IT: IntoIterator<Item = T>>(iter: IT) -> Self {
        let items = Vec::from_iter(iter);
        SequenceOf::new(items)
    }
}

impl<'a, T> TryFrom<Any<'a>> for SequenceOf<T>
where
    T: FromBer<'a>,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.tag().assert_eq(Self::TAG)?;
        if !any.header.is_constructed() {
            return Err(Error::ConstructExpected);
        }
        let items = SequenceIterator::<T, BerMode>::new(any.data.as_bytes2())
            .collect::<Result<Vec<T>>>()?;
        Ok(SequenceOf::new(items))
    }
}

impl<'i, T> BerParser<'i> for SequenceOf<T>
where
    T: BerParser<'i>,
    <T as BerParser<'i>>::Error: From<BerError<Input<'i>>>,
{
    type Error = <T as BerParser<'i>>::Error;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, items) = <Vec<T>>::from_ber_content(header, input)?;
        // NOTE: can't use SequenceIterator, it does not return `rem`

        Ok((rem, SequenceOf::new(items)))
    }
}

impl<'i, T> DerParser<'i> for SequenceOf<T>
where
    T: DerParser<'i>,
    <T as DerParser<'i>>::Error: From<BerError<Input<'i>>>,
{
    type Error = <T as DerParser<'i>>::Error;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, items) = <Vec<T>>::from_der_content(header, input)?;
        // NOTE: can't use SequenceIterator, it does not return `rem`

        Ok((rem, SequenceOf::new(items)))
    }
}

impl<T> CheckDerConstraints for SequenceOf<T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        for item in SequenceIterator::<Any, DerMode>::new(any.data.as_bytes2()) {
            let item = item?;
            T::check_constraints(&item)?;
        }
        Ok(())
    }
}

/// manual impl of FromDer, so we do not need to require `TryFrom<Any> + CheckDerConstraints`
impl<'a, T, E> FromDer<'a, E> for SequenceOf<T>
where
    T: FromDer<'a, E>,
    E: From<Error> + Display + Debug,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        trace_generic(
            core::any::type_name::<Self>(),
            "SequenceOf::from_der",
            |bytes| {
                let (rem, any) = trace(
                    core::any::type_name::<Self>(),
                    wrap_ber_parser(parse_der_any),
                    bytes,
                )
                .map_err(Err::convert)?;
                any.header
                    .assert_tag(Self::TAG)
                    .map_err(|e| Err::Error(e.into()))?;
                let items = SequenceIterator::<T, DerMode, E>::new(any.data.as_bytes2())
                    .collect::<Result<Vec<T>, E>>()
                    .map_err(Err::Error)?;
                Ok((rem, SequenceOf::new(items)))
            },
            bytes,
        )
    }
}

impl<T> Tagged for SequenceOf<T> {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::Sequence;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl<T> ToBer for SequenceOf<T>
    where
        T: ToBer + DynTagged,
    {
        type Encoder = Constructed;

        fn ber_content_len(&self) -> Length {
            self.items.ber_content_len()
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.items.ber_write_content(target)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }

    impl<T> ToDer for SequenceOf<T>
    where
        T: ToDer + DynTagged,
    {
        type Encoder = Constructed;

        fn der_content_len(&self) -> Length {
            self.items.der_content_len()
        }

        fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.items.der_write_content(target)
        }

        fn der_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, Input, SequenceOf};
    use core::iter::FromIterator;

    /// Test use of object, available methods and syntax for different use cases
    #[test]
    fn use_sequence_of() {
        let mut set = SequenceOf::from_iter([1, 2, 3]);
        set.push(4);

        // deref as slice
        let sum: i32 = set.iter().sum();
        assert_eq!(sum, 10);

        // range operator
        assert_eq!(&set[1..3], &[2, 3]);
    }

    #[test]
    fn ber_parser_sequence_of_vec() {
        // Ok: empty
        let input = Input::from_slice(&hex!("30 00"));
        let (rem, result) = <SequenceOf<u32>>::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(result.as_ref().is_empty());

        // Ok: 1 item, correct tag
        let input = Input::from_slice(&hex!("30 05 0203010001"));
        let (rem, result) = <SequenceOf<u32>>::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), &[65537]);

        // Fail: 2 items, 1 with incorrect tag
        let input = Input::from_slice(&hex!("30 08 02 03 01 00 01 01 01 ff"));
        let _ = <SequenceOf<u32>>::parse_ber(input).expect_err("parsing should have failed");

        // Fail: incomplete
        let input = Input::from_slice(&hex!("30 07 0203010001 0201"));
        let _ = <SequenceOf<u32>>::parse_ber(input).expect_err("parsing should have failed");
    }
}
