use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

// 0x12: numericstring - ASCII string with digits an spaces only
#[derive(Debug, PartialEq)]
pub struct NumericString<'a> {
    data: Cow<'a, str>,
}

impl<'a> NumericString<'a> {
    pub const fn new(s: &'a str) -> Self {
        NumericString {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(&self) -> String {
        self.data.to_string()
    }
}

impl<'a> AsRef<str> for NumericString<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for NumericString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<NumericString<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_numeric(b: &u8) -> bool {
            matches!(*b, b'0'..=b'9' | b' ')
        }
        if !any.data.iter().all(is_numeric) {
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
        Ok(NumericString { data })
    }
}

impl<'a> CheckDerConstraints for NumericString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for NumericString<'a> {
    const TAG: Tag = Tag::NumericString;
}
