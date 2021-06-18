use super::{Implicit, TaggedValue};
use crate::{
    Any, CheckDerConstraints, Error, FromBer, FromDer, Header, ParseResult, Result, Tagged,
};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::marker::PhantomData;

impl<'a, T> FromBer<'a> for TaggedValue<'a, Implicit, T>
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: Tagged,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        let Any { header, data } = any;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..header.clone()
            },
            data,
        };
        match T::try_from(any) {
            Ok(t) => {
                let tagged_value = TaggedValue {
                    header,
                    inner: t,
                    tag_kind: PhantomData,
                };
                Ok((rem, tagged_value))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a, T> FromDer<'a> for TaggedValue<'a, Implicit, T>
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: CheckDerConstraints,
    T: Tagged,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_der(bytes)?;
        let Any { header, data } = any;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..header.clone()
            },
            data,
        };
        T::check_constraints(&any)?;
        match T::try_from(any) {
            Ok(t) => {
                let tagged_value = TaggedValue {
                    header,
                    inner: t,
                    tag_kind: PhantomData,
                };
                Ok((rem, tagged_value))
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl<'a, T> CheckDerConstraints for TaggedValue<'a, Implicit, T>
where
    T: CheckDerConstraints,
    T: Tagged,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..any.header.clone()
            },
            data: Cow::Borrowed(&any.data),
        };
        T::check_constraints(&any)?;
        Ok(())
    }
}
