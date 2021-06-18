use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

// 0x14: t61string - ISO 2022 string with a Teletex (T.61) charset,
// ASCII is possible but only when explicit escaped, as by default
// the G0 character range (0x20-0x7f) will match the graphic character
// set. https://en.wikipedia.org/wiki/ITU_T.61
#[derive(Debug, PartialEq)]
pub struct TeletexString<'a> {
    data: Cow<'a, str>,
}

impl<'a> TeletexString<'a> {
    pub const fn new(s: &'a str) -> Self {
        TeletexString {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(&self) -> String {
        self.data.to_string()
    }
}

impl<'a> AsRef<str> for TeletexString<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for TeletexString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<TeletexString<'a>> {
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
        Ok(TeletexString { data })
    }
}

impl<'a> CheckDerConstraints for TeletexString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for TeletexString<'a> {
    const TAG: Tag = Tag::TeletexString;
}
