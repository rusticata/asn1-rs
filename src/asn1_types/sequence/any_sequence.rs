use alloc::vec::Vec;
use core::iter::FromIterator;
use nom::Err;

use crate::{
    Any, AnyIterator, BerError, BerMode, BerParser, DerMode, DerParser, Input, Tag, Tagged,
};

/// The `SEQUENCE` object is an ordered list of heteregeneous types.
///
/// This objects parses all items as `Any`.
#[derive(Debug)]
pub struct AnySequence<'a> {
    items: Vec<Any<'a>>,
}

impl<'a> AnySequence<'a> {
    /// Create a new `AnySequence` object.
    pub const fn new(items: Vec<Any<'a>>) -> Self {
        Self { items }
    }

    /// Returns the number of elements in the sequence
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true`` if the sequence contains no elements
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Return an iterator over the sequence objects
    pub fn iter(&self) -> impl Iterator<Item = &Any<'a>> {
        self.items.iter()
    }

    /// Return an iterator over the sequence objects (mutable)
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Any<'a>> {
        self.items.iter_mut()
    }
}

impl Tagged for AnySequence<'_> {
    const TAG: Tag = Tag::Sequence;
}

impl<'a> BerParser<'a> for AnySequence<'a> {
    type Error = BerError<Input<'a>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Self::TAG
    }

    fn from_ber_content(
        header: &'_ crate::Header<'a>,
        input: Input<'a>,
    ) -> nom::IResult<Input<'a>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.9.1)
        header
            .assert_constructed_input(&input)
            .map_err(Err::Error)?;

        let (rem, items) = AnyIterator::<BerMode>::new(input).try_collect::<Vec<Any>>()?;
        Ok((rem, Self { items }))
    }
}

impl<'a> DerParser<'a> for AnySequence<'a> {
    type Error = BerError<Input<'a>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Self::TAG
    }

    fn from_der_content(
        header: &'_ crate::Header<'a>,
        input: Input<'a>,
    ) -> nom::IResult<Input<'a>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.9.1)
        header
            .assert_constructed_input(&input)
            .map_err(Err::Error)?;

        let (rem, items) = AnyIterator::<DerMode>::new(input).try_collect::<Vec<Any>>()?;
        Ok((rem, Self { items }))
    }
}

impl<'a> FromIterator<Any<'a>> for AnySequence<'a> {
    fn from_iter<T: IntoIterator<Item = Any<'a>>>(iter: T) -> Self {
        let items = iter.into_iter().collect();
        Self { items }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, Input};

    use super::AnySequence;

    #[test]
    fn parse_ber_anysequence() {
        let input = &hex!("30 05 02 03 01 00 01");
        let (rem, result) = <AnySequence>::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.len(), 1);

        assert_eq!(result.iter().next().unwrap().as_u32(), Ok(65537));
    }
}
