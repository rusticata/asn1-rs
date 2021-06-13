use crate::{ber::bytes_to_u64, Any, CheckDerConstraints, Error, Result, Tag, Tagged};
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Enumerated(pub u32);

impl Enumerated {
    pub const fn new(value: u32) -> Self {
        Enumerated(value)
    }
}

impl<'a> TryFrom<Any<'a>> for Enumerated {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Enumerated> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        let res_u64 = bytes_to_u64(&any.data)?;
        if res_u64 > (<u32>::MAX as u64) {
            return Err(Error::IntegerTooLarge);
        }
        let value = res_u64 as u32;
        Ok(Enumerated(value))
    }
}

impl CheckDerConstraints for Enumerated {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl Tagged for Enumerated {
    const TAG: Tag = Tag::Enumerated;
}
