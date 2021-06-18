use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct VisibleString<'a> {
    data: Cow<'a, str>,
}

impl<'a> VisibleString<'a> {
    pub const fn new(s: &'a str) -> Self {
        VisibleString {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(&self) -> String {
        self.data.to_string()
    }
}

impl<'a> AsRef<str> for VisibleString<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for VisibleString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<VisibleString<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_visible(b: &u8) -> bool {
            0x20 <= *b && *b <= 0x7f
        }
        if !any.data.iter().all(is_visible) {
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
        Ok(VisibleString { data })
    }
}

impl<'a> CheckDerConstraints for VisibleString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for VisibleString<'a> {
    const TAG: Tag = Tag::VisibleString;
}
