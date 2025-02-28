use nom::IResult;

use crate::{
    Any, BerError, BerParser, CheckDerConstraints, DerAutoDerive, DerParser, Error, Header, Input,
    Real, Result, Tag, Tagged,
};
use core::convert::{TryFrom, TryInto};

impl<'a> TryFrom<Any<'a>> for f32 {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<f32> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        let real: Real = any.try_into()?;
        Ok(real.f32())
    }
}

impl<'i> BerParser<'i> for f32 {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::RealType
    }

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, real) = Real::from_ber_content(header, input)?;

        Ok((rem, real.f32()))
    }
}

impl<'i> DerParser<'i> for f32 {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::RealType
    }

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, real) = Real::from_der_content(header, input)?;

        Ok((rem, real.f32()))
    }
}

impl CheckDerConstraints for f32 {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl DerAutoDerive for f32 {}

impl Tagged for f32 {
    const TAG: Tag = Tag::RealType;
}
