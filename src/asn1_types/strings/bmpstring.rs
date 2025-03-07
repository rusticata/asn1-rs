// do not use the `asn1_string` macro, since types are not the same
// X.680 section 37.15

use crate::*;
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use nom::Input as _;

/// ASN.1 `BMPSTRING` type
///
/// Note: parsing a `BmpString` allocates memory since the UTF-16 to UTF-8 conversion requires a memory allocation.
/// (see `String::from_utf16` method).
#[derive(Debug, PartialEq, Eq)]
pub struct BmpString<'a> {
    pub(crate) data: Cow<'a, str>,
}

impl<'a> BmpString<'a> {
    pub const fn new(s: &'a str) -> Self {
        BmpString {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(&self) -> String {
        self.data.to_string()
    }
}

impl AsRef<str> for BmpString<'_> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> From<&'a str> for BmpString<'a> {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl From<String> for BmpString<'_> {
    fn from(s: String) -> Self {
        Self {
            data: Cow::Owned(s),
        }
    }
}

impl_tryfrom_any!('i @ BmpString<'i>);

impl<'i> BerParser<'i> for BmpString<'i> {
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

        // read slice as big-endian UTF-16 string
        let v = data
            .as_ref()
            .chunks(2)
            .map(|s| match s {
                [a, b] => ((*a as u16) << 8) | (*b as u16),
                [a] => *a as u16,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let s = String::from_utf16(&v)
            .map_err(|_| BerError::nom_err_input(&rem, InnerError::StringInvalidCharset))?;
        let data = Cow::Owned(s);

        Ok((rem, BmpString { data }))
    }
}

impl<'i> DerParser<'i> for BmpString<'i> {
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

impl CheckDerConstraints for BmpString<'_> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl DerAutoDerive for BmpString<'_> {}

impl Tagged for BmpString<'_> {
    const TAG: Tag = Tag::BmpString;
}

impl TestValidCharset for BmpString<'_> {
    fn test_valid_charset(i: &[u8]) -> Result<()> {
        if i.len() % 2 != 0 {
            return Err(Error::StringInvalidCharset);
        }
        let iter = i.chunks(2).map(|s| ((s[0] as u16) << 8) | (s[1] as u16));
        for c in char::decode_utf16(iter) {
            if c.is_err() {
                return Err(Error::StringInvalidCharset);
            }
        }
        Ok(())
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for BmpString<'_> {
        type Encoder = Primitive<{ Tag::BmpString.0 }>;

        fn ber_content_len(&self) -> Length {
            // compute the UTF-16 length
            let sz = self.data.encode_utf16().count() * 2;
            Length::Definite(sz)
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let mut v = Vec::new();
            for u in self.data.encode_utf16() {
                v.push((u >> 8) as u8);
                v.push((u & 0xff) as u8);
            }
            target.write(&v).map_err(Into::into)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(LFT 'a, BmpString<'a>);
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, BmpString, DerParser};

    #[test]
    fn parse_ber_bmpstring() {
        // taken from https://docs.microsoft.com/en-us/windows/win32/seccertenroll/about-bmpstring
        let input = &hex!("1e 08 00 55 00 73 00 65 00 72");
        let (rem, result) = BmpString::parse_ber(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "User");
    }

    #[test]
    fn parse_der_bmpstring() {
        // taken from https://docs.microsoft.com/en-us/windows/win32/seccertenroll/about-bmpstring
        let input = &hex!("1e 08 00 55 00 73 00 65 00 72");
        let (rem, result) = BmpString::parse_der(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "User");
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{BmpString, ToBer};

        #[test]
        fn tober_bmpstring() {
            let s = BmpString::new("User");
            let mut v: Vec<u8> = Vec::new();
            s.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"1e 08 0055 0073 0065 0072"});
        }
    }
}
