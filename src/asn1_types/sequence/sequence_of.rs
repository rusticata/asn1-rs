use crate::{
    BerParser, DerParser, FromBer, FromDer, ParseResult, Result, Sequence, SequenceIterator, ToDer,
};
use std::borrow::Cow;

#[derive(Debug)]
pub struct SequenceOf<T> {
    pub(crate) items: Vec<T>,
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

impl<T> ToDer for SequenceOf<T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        self.items.to_der_len()
    }

    fn to_der(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        self.items.to_der(writer)
    }
}
