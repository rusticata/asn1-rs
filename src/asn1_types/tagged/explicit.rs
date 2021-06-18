use super::{Explicit, TaggedValue};
use crate::{Any, CheckDerConstraints, FromBer, FromDer, ParseResult, Result};
use std::borrow::Cow;
use std::marker::PhantomData;

impl<'a, T> FromBer<'a> for TaggedValue<'a, Explicit, T>
where
    T: FromBer<'a>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        let header = any.header;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            Cow::Owned(_) => unreachable!(),
        };
        let (_, inner) = T::from_ber(data)?;
        let tagged = TaggedValue {
            header,
            inner,
            tag_kind: PhantomData,
        };
        Ok((rem, tagged))
    }
}

impl<'a, T> FromDer<'a> for TaggedValue<'a, Explicit, T>
where
    T: FromDer<'a>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_der(bytes)?;
        let header = any.header;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            Cow::Owned(_) => unreachable!(),
        };
        let (_, inner) = T::from_der(data)?;
        let tagged = TaggedValue {
            header,
            inner,
            tag_kind: PhantomData,
        };
        Ok((rem, tagged))
    }
}

impl<'a, T> CheckDerConstraints for TaggedValue<'a, Explicit, T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let (_, inner_any) = Any::from_der(&any.data)?;
        T::check_constraints(&inner_any)?;
        Ok(())
    }
}
