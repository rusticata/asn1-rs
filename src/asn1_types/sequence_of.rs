use crate::{
    Any, BerParser, DerParser, FromBer, FromDer, ParseResult, Result, Sequence, SequenceIterator,
    Tag, Tagged,
};
use std::borrow::Cow;

#[derive(Debug)]
pub struct SequenceOf<T> {
    items: Vec<T>,
}

impl<T> SequenceOf<T> {
    pub const fn new(items: Vec<T>) -> Self {
        SequenceOf { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<'a, T> AsRef<[T]> for SequenceOf<T> {
    fn as_ref(&self) -> &[T] {
        &self.items
    }
}

impl<'a, T> FromBer<'a> for SequenceOf<T>
where
    T: FromBer<'a>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, seq) = Sequence::from_ber(bytes)?;
        let data = match seq.content {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SequenceIterator::<T, BerParser>::new(data).collect::<Result<Vec<T>>>()?;
        Ok((rem, SequenceOf::new(v)))
    }
}

impl<'a, T> FromDer<'a> for SequenceOf<T>
where
    T: FromDer<'a>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, seq) = Sequence::from_der(bytes)?;
        let data = match seq.content {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SequenceIterator::<T, DerParser>::new(data).collect::<Result<Vec<T>>>()?;
        Ok((rem, SequenceOf::new(v)))
    }
}

impl<T> From<SequenceOf<T>> for Vec<T> {
    fn from(set: SequenceOf<T>) -> Self {
        set.items
    }
}

impl<T> Tagged for SequenceOf<T> {
    const TAG: Tag = Tag::Sequence;
}

impl<T> Tagged for Vec<T> {
    const TAG: Tag = Tag::Sequence;
}

impl<'a, T> FromBer<'a> for Vec<T>
where
    T: FromBer<'a>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        any.header.assert_tag(Self::TAG)?;
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
        any.header.assert_tag(Self::TAG)?;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SequenceIterator::<T, DerParser>::new(data).collect::<Result<Vec<T>>>()?;
        Ok((rem, v))
    }
}
