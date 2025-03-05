use nom::IResult;

use crate::{
    impl_tryfrom_any, Any, BerError, BerParser, CheckDerConstraints, DerAutoDerive, DerParser,
    Header, Input, Real, Result, Tag, Tagged,
};

impl_tryfrom_any!(f64);

impl<'i> BerParser<'i> for f64 {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, real) = Real::from_ber_content(header, input)?;

        Ok((rem, real.f64()))
    }
}

impl<'i> DerParser<'i> for f64 {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, real) = Real::from_der_content(header, input)?;

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

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    use crate::{Length, Primitive, SerializeResult, ToBer};

    impl ToBer for f64 {
        type Encoder = Primitive<Self, { Tag::RealType.0 }>;

        fn content_len(&self) -> Length {
            let r = Real::from(*self);
            r.content_len()
        }

        fn write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let r = Real::from(*self);
            r.write_content(target)
        }
    }
};
