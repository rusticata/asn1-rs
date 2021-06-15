use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct GeneralString<'a> {
    data: Cow<'a, str>,
}

impl<'a> GeneralString<'a> {
    pub const fn new(s: &'a str) -> Self {
        GeneralString {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(&self) -> String {
        self.data.to_string()
    }
}

impl<'a> AsRef<str> for GeneralString<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for GeneralString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<GeneralString<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        if !any.data.iter().all(u8::is_ascii) {
            return Err(Error::StringInvalidCharset);
        }
        let data = match any.data {
            Cow::Borrowed(b) => {
                let s = std::str::from_utf8(b)?;
                Cow::Borrowed(s)
            }
            Cow::Owned(v) => {
                let s = std::string::String::from_utf8(v)?;
                Cow::Owned(s)
            }
        };
        Ok(GeneralString { data })
    }
}

impl<'a> CheckDerConstraints for GeneralString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for GeneralString<'a> {
    const TAG: Tag = Tag::GeneralString;
}
