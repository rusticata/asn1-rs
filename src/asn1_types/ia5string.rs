use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct IA5String<'a> {
    data: Cow<'a, str>,
}

impl<'a> IA5String<'a> {
    pub const fn new(s: &'a str) -> Self {
        IA5String {
            data: Cow::Borrowed(s),
        }
    }
}

impl<'a> AsRef<str> for IA5String<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for IA5String<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<IA5String<'a>> {
        any.tag().assert_eq(Self::TAG)?;
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
        Ok(IA5String { data })
    }
}

impl<'a> CheckDerConstraints for IA5String<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for IA5String<'a> {
    const TAG: Tag = Tag::Ia5String;
}
