use crate::{
    Any, BerParser, DerParser, FromBer, FromDer, ParseResult, Result, SetIterator, Tag, Tagged,
};
use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::Hash;

impl<T> Tagged for HashSet<T> {
    const TAG: Tag = Tag::Set;
}

impl<'a, T> FromBer<'a> for HashSet<T>
where
    T: FromBer<'a>,
    T: Hash + Eq,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        any.header.assert_tag(Self::TAG)?;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SetIterator::<T, BerParser>::new(data).collect::<Result<HashSet<T>>>()?;
        Ok((rem, v))
    }
}

impl<'a, T> FromDer<'a> for HashSet<T>
where
    T: FromDer<'a>,
    T: Hash + Eq,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<Self> {
        let (rem, any) = Any::from_der(bytes)?;
        any.header.assert_tag(Self::TAG)?;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SetIterator::<T, DerParser>::new(data).collect::<Result<HashSet<T>>>()?;
        Ok((rem, v))
    }
}
