#![cfg(feature = "std")]

use core::iter::FromIterator;
use nom::Err;
use std::collections::HashSet;

use crate::{
    Any, AnyIterator, BerError, BerMode, BerParser, DerMode, DerParser, Input, Tag, Tagged,
};

/// The `SET` object is an unordered list of heteregeneous types.
///
/// This objects parses all items as `Any`.
///
/// Items in set must be unique. Any attempt to insert an object twice will overwrite the
/// previous object.
/// This is enforced by using a hash function internally.
#[derive(Debug)]
pub struct AnySet<'a> {
    items: HashSet<Any<'a>>,
}

impl<'a> AnySet<'a> {
    /// Create a new `AnySequence` object.
    ///
    /// See also the [`FromIterator`] trait, implemented for this type.
    pub const fn new(items: HashSet<Any<'a>>) -> Self {
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

    /// Add an item to the set
    pub fn insert(&mut self, item: Any<'a>) -> bool {
        self.items.insert(item)
    }

    /// Remove an object from the set
    pub fn remove(&mut self, item: &Any<'a>) -> bool {
        self.items.remove(item)
    }
}

impl Tagged for AnySet<'_> {
    const TAG: Tag = Tag::Set;
}

impl<'a> BerParser<'a> for AnySet<'a> {
    type Error = BerError<Input<'a>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Self::TAG
    }

    fn from_ber_content(
        header: &'_ crate::Header<'a>,
        input: Input<'a>,
    ) -> nom::IResult<Input<'a>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.11.1)
        header
            .assert_constructed_input(&input)
            .map_err(Err::Error)?;

        let (rem, items) = AnyIterator::<BerMode>::new(input).try_collect::<HashSet<Any>>()?;
        Ok((rem, Self { items }))
    }
}

impl<'a> DerParser<'a> for AnySet<'a> {
    type Error = BerError<Input<'a>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Self::TAG
    }

    fn from_der_content(
        header: &'_ crate::Header<'a>,
        input: Input<'a>,
    ) -> nom::IResult<Input<'a>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.11.1)
        header
            .assert_constructed_input(&input)
            .map_err(Err::Error)?;

        let (rem, items) = AnyIterator::<DerMode>::new(input).try_collect::<HashSet<Any>>()?;
        Ok((rem, Self { items }))
    }
}

impl<'a> FromIterator<Any<'a>> for AnySet<'a> {
    fn from_iter<T: IntoIterator<Item = Any<'a>>>(iter: T) -> Self {
        let items = iter.into_iter().collect();
        Self { items }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, DerParser, Input};

    use super::AnySet;

    #[test]
    fn parse_ber_anyset() {
        let input = &hex!("31 05 02 03 01 00 01");
        let (rem, result) = <AnySet>::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.len(), 1);

        assert_eq!(result.iter().next().unwrap().as_u32(), Ok(65537));
        // Ok: indefinite length
        let input = &hex!("31 80 0203010001 0000");
        let (rem, result) = <AnySet>::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.len(), 1);

        assert_eq!(result.iter().next().unwrap().as_u32(), Ok(65537));
    }

    #[test]
    fn parse_der_anyset() {
        let input = &hex!("31 05 02 03 01 00 01");
        let (rem, result) = <AnySet>::parse_der(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.len(), 1);

        assert_eq!(result.iter().next().unwrap().as_u32(), Ok(65537));

        // Fail: indefinite length
        let input = &hex!("31 80 0203010001 0000");
        let _ = <AnySet>::parse_der(Input::from(input)).expect_err("indefinite length");
    }
}
