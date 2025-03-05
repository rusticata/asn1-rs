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

#[cfg(feature = "std")]
const _: () = {
    use std::io;
    use std::io::Write;

    use crate::{ber_header_length, Constructed, Length, ToBer};

    impl ToBer for AnySet<'_> {
        type Encoder = Constructed<Self>;

        fn content_len(&self) -> Length {
            // content_len returns only the length of *content*, so we need header length for
            // every object here
            let len = self.iter().fold(Length::Definite(0), |acc, t| {
                let content_length = t.content_len();
                match (acc, content_length) {
                    (Length::Definite(a), Length::Definite(b)) => {
                        let header_length = ber_header_length(t.tag(), content_length).unwrap_or(0);
                        Length::Definite(a + header_length + b)
                    }
                    _ => Length::Indefinite,
                }
            });
            len
        }

        fn write_content<W: Write>(&self, target: &mut W) -> Result<usize, io::Error> {
            self.iter().try_fold(0, |acc, t| {
                let sz = t.encode(target)?;
                Ok::<_, io::Error>(acc + sz)
            })
        }
    }
};

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

    #[cfg(feature = "std")]
    mod tests_std {
        use core::iter::FromIterator;

        use hex_literal::hex;

        use crate::{Any, AnySet, Tag, ToBer};

        #[test]
        fn tober_anyset() {
            let v = &[
                Any::from_tag_and_data(Tag::OctetString, (&hex!("01020304")).into()),
                Any::from_tag_and_data(Tag::Integer, (&hex!("010001")).into()),
            ];
            let s = AnySet::from_iter(v.iter().cloned());
            let mut v: Vec<u8> = Vec::new();
            s.encode(&mut v).expect("serialization failed");
            assert_eq!(
                &v,
                &hex! {"31 0b
                0404 01020304
                0203 010001"}
            );
        }
    }
}
