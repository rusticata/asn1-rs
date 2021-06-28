// do not use the `asn1_string` macro, since types are not the same
// X.680 section 37.15

use crate::{Any, CheckDerConstraints, Class, Error, Header, Length, Result, Tag, Tagged, ToDer};
use std::borrow::Cow;

/// `BMPSTRING` ASN.1 string
///
/// Note: parsing a `BmpString` allocates memory since the UTF-16 to UTF-8 conversion requires a memory allocation.
/// (see `String::from_utf16` method).
#[derive(Debug, PartialEq)]
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

impl<'a> AsRef<str> for BmpString<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> std::convert::TryFrom<Any<'a>> for BmpString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<BmpString<'a>> {
        any.tag().assert_eq(Self::TAG)?;

        // read slice as big-endian UTF-16 string
        let v = &any
            .data
            .chunks(2)
            .map(|s| match s {
                [a, b] => ((*a as u16) << 8) | (*b as u16),
                [a] => *a as u16,
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();

        let s = std::string::String::from_utf16(&v)?;
        let data = Cow::Owned(s);

        Ok(BmpString { data })
    }
}

impl<'a> CheckDerConstraints for BmpString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for BmpString<'a> {
    const TAG: Tag = Tag::BmpString;
}

impl ToDer for BmpString<'_> {
    fn to_der_len(&self) -> Result<usize> {
        let header = Header::new(
            Class::Universal,
            0,
            Self::TAG,
            Length::Definite(self.data.as_bytes().len()),
        );
        Ok(header.to_der_len()? + self.data.as_bytes().len())
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        let header = Header::new(
            Class::Universal,
            0,
            Self::TAG,
            Length::Definite(self.data.as_bytes().len()),
        );
        header.write_der_header(writer).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        writer.write(&self.data.as_bytes()).map_err(Into::into)
    }
}
