use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::convert::TryFrom;

#[derive(Debug)]
pub struct Null {}

impl Null {
    pub const fn new() -> Self {
        Null {}
    }
}

impl<'a> TryFrom<Any<'a>> for Null {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Null> {
        any.tag().assert_eq(Self::TAG)?;
        if !any.header.length.is_null() {
            return Err(Error::InvalidLength);
        }
        Ok(Null {})
    }
}

impl<'a> CheckDerConstraints for Null {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl<'a> Tagged for Null {
    const TAG: Tag = Tag::Null;
}

impl<'a> TryFrom<Any<'a>> for () {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<()> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        if !any.header.length.is_null() {
            return Err(Error::InvalidLength);
        }
        Ok(())
    }
}

impl<'a> CheckDerConstraints for () {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl<'a> Tagged for () {
    const TAG: Tag = Tag::Null;
}
