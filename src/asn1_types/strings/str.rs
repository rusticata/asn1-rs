use crate::{Any, CheckDerConstraints, Error, Result, Tag, Tagged, Utf8String};
use std::borrow::Cow;
use std::convert::TryFrom;

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
