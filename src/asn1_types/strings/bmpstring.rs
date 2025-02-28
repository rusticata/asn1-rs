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

impl<'a, 'r> core::convert::TryFrom<&'r Any<'a>> for BmpString<'a> {
    type Error = Error;

    fn try_from(any: &'r Any<'a>) -> Result<BmpString<'a>> {
        any.tag().assert_eq(Self::TAG)?;

        // read slice as big-endian UTF-16 string
        let v = &any
            .data
            .as_bytes2()
            .chunks(2)
            .map(|s| match s {
                [a, b] => ((*a as u16) << 8) | (*b as u16),
                [a] => *a as u16,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let s = String::from_utf16(v)?;
        let data = Cow::Owned(s);

        Ok(BmpString { data })
    }
}

impl<'i> BerParser<'i> for BmpString<'i> {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::BmpString
    }

    fn from_any_ber(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
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

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::BmpString
    }

    fn from_any_der(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        Self::from_any_ber(input, header)
    }
}

impl<'a> core::convert::TryFrom<Any<'a>> for BmpString<'a> {
    type Error = Error;

    #[inline]
    fn try_from(any: Any<'a>) -> Result<BmpString<'a>> {
        BmpString::try_from(&any)
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
impl ToDer for BmpString<'_> {
    fn to_der_len(&self) -> Result<usize> {
        // compute the UTF-16 length
        let sz = self.data.encode_utf16().count() * 2;
        if sz < 127 {
            // 1 (class+tag) + 1 (length) + len
            Ok(2 + sz)
        } else {
            // 1 (class+tag) + n (length) + len
            let n = Length::Definite(sz).to_der_len()?;
            Ok(1 + n + sz)
        }
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        // compute the UTF-16 length
        let l = self.data.encode_utf16().count() * 2;
        let header = Header::new(Class::Universal, false, Self::TAG, Length::Definite(l));
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut v = Vec::new();
        for u in self.data.encode_utf16() {
            v.push((u >> 8) as u8);
            v.push((u & 0xff) as u8);
        }
        writer.write(&v).map_err(Into::into)
    }
}

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
}
