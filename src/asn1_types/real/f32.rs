use nom::IResult;

use crate::{
    impl_tryfrom_any, Any, BerError, BerParser, CheckDerConstraints, DerAutoDerive, DerParser,
    Header, Input, Real, Result, Tag, Tagged,
};

impl_tryfrom_any!(f32);

impl<'i> BerParser<'i> for f32 {
    type Error = BerError<Input<'i>>;

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

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    use crate::{impl_toder_from_tober, Class, Length, Primitive, SerializeResult, ToBer};

    impl ToBer for f32 {
        type Encoder = Primitive<{ Tag::RealType.0 }>;

        fn ber_content_len(&self) -> Length {
            let r = Real::from(*self);
            r.ber_content_len()
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let r = Real::from(*self);
            r.ber_write_content(target)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(TY f32);
};
