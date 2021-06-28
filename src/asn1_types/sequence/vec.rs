use super::{SequenceIterator, SequenceOf};
use crate::{
    Any, BerParser, Class, DerParser, FromBer, FromDer, Header, Length, ParseResult, Result,
    SerializeError, Tag, Tagged, ToDer,
};
use std::borrow::Cow;

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

impl<T> ToDer for Vec<T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        let mut len = 0;
        for t in self.iter() {
            len += t.to_der_len()?;
        }
        let header = Header::new(Class::Universal, 1, Self::TAG, Length::Definite(len));
        Ok(header.to_der_len()? + len)
    }

    fn to_der(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        let mut len = 0;
        for t in self.iter() {
            len += t.to_der_len().map_err(|_| SerializeError::InvalidLength)?;
        }
        let header = Header::new(Class::Universal, 1, Self::TAG, Length::Definite(len));
        let mut sz = header.to_der(writer)?;
        for t in self.iter() {
            sz += t.to_der(writer)?;
        }
        Ok(sz)
    }
}
