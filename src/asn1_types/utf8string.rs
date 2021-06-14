use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Utf8String<'a> {
    data: Cow<'a, str>,
}

impl<'a> Utf8String<'a> {
    pub const fn new(s: &'a str) -> Self {
        Utf8String {
            data: Cow::Borrowed(s),
        }
    }

    pub fn string(self) -> String {
        self.data.into_owned()
    }
}

impl<'a> AsRef<str> for Utf8String<'a> {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for Utf8String<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Utf8String<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        if any.header.is_constructed() {
            return Err(Error::Unsupported);
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
        Ok(Utf8String { data })
    }
}

impl<'a> CheckDerConstraints for Utf8String<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for Utf8String<'a> {
    const TAG: Tag = Tag::Utf8String;
}

impl<'a> TryFrom<Any<'a>> for String {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<String> {
        any.tag().assert_eq(Self::TAG)?;
        let s = Utf8String::try_from(any)?;
        Ok(s.data.into_owned())
    }
}

impl<'a> CheckDerConstraints for String {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl Tagged for String {
    const TAG: Tag = Tag::Utf8String;
}

impl<'a> TryFrom<Any<'a>> for &'a str {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<&'a str> {
        any.tag().assert_eq(Self::TAG)?;
        let s = Utf8String::try_from(any)?;
        match s.data {
            Cow::Borrowed(s) => Ok(s),
            Cow::Owned(_) => Err(Error::LifetimeError),
        }
    }
}

impl<'a> CheckDerConstraints for &'a str {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for &'a str {
    const TAG: Tag = Tag::Utf8String;
}
