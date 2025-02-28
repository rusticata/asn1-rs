use nom::IResult;

use crate::{
    Any, BerError, BerParser, CheckDerConstraints, DerAutoDerive, DerParser, Error, Header, Input,
    Real, Result, Tag, Tagged,
};
use core::convert::{TryFrom, TryInto};

impl<'a> TryFrom<Any<'a>> for f64 {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<f64> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        let real: Real = any.try_into()?;
        Ok(real.f64())
    }
}

impl<'i> BerParser<'i> for f64 {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::RealType
    }

    fn from_ber_content(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, real) = Real::from_ber_content(input, header)?;

        Ok((rem, real.f64()))
    }
}

impl<'i> DerParser<'i> for f64 {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::RealType
    }

    fn from_der_content(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, real) = Real::from_der_content(input, header)?;

        Ok((rem, real.f64()))
    }
}

impl CheckDerConstraints for f64 {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl DerAutoDerive for f64 {}

impl Tagged for f64 {
    const TAG: Tag = Tag::RealType;
}
