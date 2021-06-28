use crate::{
    Any, BerParser, Class, DerParser, FromBer, FromDer, Header, Length, ParseResult, Result,
    SerializeError, SetIterator, Tag, Tagged, ToDer,
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

impl<T> ToDer for HashSet<T>
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

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        let mut len = 0;
        for t in self.iter() {
            len += t.to_der_len().map_err(|_| SerializeError::InvalidLength)?;
        }
        let header = Header::new(Class::Universal, 1, Self::TAG, Length::Definite(len));
        header.write_der_header(writer).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> crate::SerializeResult<usize> {
        let mut sz = 0;
        for t in self.iter() {
            sz += t.write_der(writer)?;
        }
        Ok(sz)
    }
}
