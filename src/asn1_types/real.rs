use crate::{Any, CheckDerConstraints, Error, Result, Tag, Tagged};
use std::convert::TryFrom;

impl<'a> TryFrom<Any<'a>> for f32 {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<f32> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        // let result = any.data[0];
        // Ok(result)
        unimplemented!("Support for REAL not yet implemented")
    }
}

impl<'a> CheckDerConstraints for f32 {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl Tagged for f32 {
    const TAG: Tag = Tag::RealType;
}

impl<'a> TryFrom<Any<'a>> for f64 {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<f64> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        // let result = any.data[0];
        // Ok(result)
        unimplemented!("Support for REAL not yet implemented")
    }
}

impl<'a> CheckDerConstraints for f64 {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl Tagged for f64 {
    const TAG: Tag = Tag::RealType;
}
