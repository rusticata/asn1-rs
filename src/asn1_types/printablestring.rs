use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct PrintableString<'a> {
    data: Cow<'a, str>,
}

impl<'a> PrintableString<'a> {
    pub const fn new(s: &'a str) -> Self {
        PrintableString {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(self) -> String {
        self.data.into_owned()
    }
}

impl<'a> AsRef<str> for PrintableString<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for PrintableString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<PrintableString<'a>> {
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
        Ok(PrintableString { data })
    }
}

impl<'a> CheckDerConstraints for PrintableString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for PrintableString<'a> {
    const TAG: Tag = Tag::PrintableString;
}
