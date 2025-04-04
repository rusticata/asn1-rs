// do not use the `asn1_string` macro, since types are not the same
// X.680 section 37.6 and X.690 section 8.21.7

use crate::*;
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::iter::FromIterator;
use nom::Input as _;

/// ASN.1 `UniversalString` type
///
/// Note: parsing a `UniversalString` allocates memory since the UCS-4 to UTF-8 conversion requires a memory allocation.
#[derive(Debug, PartialEq, Eq)]
pub struct UniversalString<'a> {
    pub(crate) data: Cow<'a, str>,
}

impl<'a> UniversalString<'a> {
    pub const fn new(s: &'a str) -> Self {
        UniversalString {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(&self) -> String {
        self.data.to_string()
    }
}

impl AsRef<str> for UniversalString<'_> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> From<&'a str> for UniversalString<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl From<String> for UniversalString<'_> {
    fn from(s: String) -> Self {
        Self {
            data: Cow::Owned(s),
        }
    }
}

impl_tryfrom_any!('i @ UniversalString<'i>);

impl<'i> BerParser<'i> for UniversalString<'i> {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall either be primitive or constructed (X.690: 8.20)
        let (rem, data) = if header.is_constructed() {
            let (rem, data) = input.take_split(input.len());
            (rem, Cow::Borrowed(data.as_bytes2()))
        } else {
            let (rem, s) =
                parse_ber_segmented::<OctetString>(header, input, OCTETSTRING_MAX_RECURSION)?;
            let s = s.into_cow();
            (rem, s)
        };

        if data.len() % 4 != 0 {
            return Err(BerError::nom_err_input(
                &rem,
                InnerError::StringInvalidCharset,
            ));
        }

        // read slice as big-endian UCS-4 string
        let v = data
            .as_ref()
            .chunks(4)
            .map(|s| match s {
                [a, b, c, d] => {
                    let u32_val = ((*a as u32) << 24)
                        | ((*b as u32) << 16)
                        | ((*c as u32) << 8)
                        | (*d as u32);
                    char::from_u32(u32_val)
                }
                _ => unreachable!(),
            })
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| BerError::nom_err_input(&rem, InnerError::StringInvalidCharset))?;

        let s = String::from_iter(v);
        let data = Cow::Owned(s);

        Ok((rem, UniversalString { data }))
    }
}

impl<'i> DerParser<'i> for UniversalString<'i> {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        Self::from_ber_content(header, input)
    }
}

impl CheckDerConstraints for UniversalString<'_> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl DerAutoDerive for UniversalString<'_> {}

impl Tagged for UniversalString<'_> {
    const TAG: Tag = Tag::UniversalString;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for UniversalString<'_> {
        type Encoder = Primitive<{ Tag::UniversalString.0 }>;

        fn ber_content_len(&self) -> Length {
            // UCS-4: 4 bytes per character
            let sz = self.data.len() * 4;
            Length::Definite(sz)
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.data
                .chars()
                .try_for_each(|c| target.write(&(c as u32).to_be_bytes()[..]).map(|_| ()))?;
            Ok(self.data.len() * 4)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(LFT 'a, UniversalString<'a>);
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, DerParser, UniversalString};

    #[test]
    fn parse_ber_universalstring() {
        let input = &hex!("1C 10 00000061 00000062 00000063 00000064");
        let (rem, result) = UniversalString::parse_ber(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "abcd");
    }

    #[test]
    fn parse_der_universalstring() {
        let input = &hex!("1C 10 00000061 00000062 00000063 00000064");
        let (rem, result) = UniversalString::parse_der(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "abcd");
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{ToBer, UniversalString};

        #[test]
        fn tober_universalstring() {
            let s = UniversalString::new("abcd");
            let mut v: Vec<u8> = Vec::new();
            s.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"1C 10 00000061 00000062 00000063 00000064"});
        }
    }
}
