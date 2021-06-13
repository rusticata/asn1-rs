use crate::{Any, Error, Result, Tag, Tagged};
use crate::{CheckDerConstraints, Length};
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Boolean {
    pub value: u8,
}

impl Boolean {
    pub const FALSE: Boolean = Boolean::new(0);
    pub const TRUE: Boolean = Boolean::new(0xff);

    #[inline]
    pub const fn new(value: u8) -> Self {
        Boolean { value }
    }

    #[inline]
    pub const fn bool(&self) -> bool {
        self.value != 0
    }
}

impl<'a> TryFrom<Any<'a>> for Boolean {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Boolean> {
        any.tag().assert_eq(Self::TAG)?;
        // X.690 section 8.2.1:
        // The encoding of a boolean value shall be primitive. The contents octets shall consist of a single octet
        if any.header.length != Length::Definite(1) {
            return Err(Error::InvalidLength);
        }
        let value = any.data[0];
        Ok(Boolean { value })
    }
}

impl<'a> CheckDerConstraints for Boolean {
    fn check_constraints(any: &Any) -> Result<()> {
        let c = any.data[0];
        // X.690 section 11.1
        if !(c == 0 || c == 0xff) {
            return Err(Error::DerConstraintFailed);
        }
        Ok(())
    }
}

impl<'a> Tagged for Boolean {
    const TAG: Tag = Tag::Boolean;
}

impl<'a> TryFrom<Any<'a>> for bool {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<bool> {
        any.tag().assert_eq(Self::TAG)?;
        let b = Boolean::try_from(any)?;
        Ok(b.bool())
    }
}

impl<'a> CheckDerConstraints for bool {
    fn check_constraints(any: &Any) -> Result<()> {
        let c = any.data[0];
        // X.690 section 11.1
        if !(c == 0 || c == 0xff) {
            return Err(Error::DerConstraintFailed);
        }
        Ok(())
    }
}

impl<'a> Tagged for bool {
    const TAG: Tag = Tag::Boolean;
}
