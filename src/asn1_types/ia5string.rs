use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Ia5String<'a> {
    data: Cow<'a, str>,
}

impl<'a> Ia5String<'a> {
    pub const fn new(s: &'a str) -> Self {
        Ia5String {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(&self) -> String {
        self.data.to_string()
    }
}

impl<'a> AsRef<str> for Ia5String<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for Ia5String<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Ia5String<'a>> {
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
        Ok(Ia5String { data })
    }
}

impl<'a> CheckDerConstraints for Ia5String<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for Ia5String<'a> {
    const TAG: Tag = Tag::Ia5String;
}
