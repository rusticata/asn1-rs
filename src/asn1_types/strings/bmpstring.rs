// do not use the `asn1_string` macro, since types are not the same
// X.680 section 37.15

use crate::{Any, CheckDerConstraints, Error, Result, Tag, Tagged};
use bytemuck::try_cast_slice;
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

        let u16_slice: &[u16] = match try_cast_slice(&any.data) {
            Ok(s) => s,
            Err(_) => return Err(Error::StringInvalidCharset),
        };

        let s = std::string::String::from_utf16(u16_slice)?;
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
