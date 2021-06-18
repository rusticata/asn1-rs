use crate::{Any, CheckDerConstraints, Error, Result, Tag, Tagged, Utf8String};
use std::convert::TryFrom;

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
