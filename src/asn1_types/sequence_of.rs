use crate::{
    Any, BerParser, DerParser, FromBer, FromDer, ParseResult, Result, SequenceIterator, Tag, Tagged,
};
use std::borrow::Cow;

impl<T> Tagged for Vec<T> {
    const TAG: Tag = Tag::Sequence;
}

impl<'a, T> FromBer<'a> for Vec<T>
where
    T: FromBer<'a>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SequenceIterator::<T, BerParser>::new(data).collect::<Result<Vec<T>>>()?;
        Ok((rem, v))
    }
}

impl<'a, T> FromDer<'a> for Vec<T>
where
    T: FromDer<'a>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<Self> {
        let (rem, any) = Any::from_der(bytes)?;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SequenceIterator::<T, DerParser>::new(data).collect::<Result<Vec<T>>>()?;
        Ok((rem, v))
    }
}
