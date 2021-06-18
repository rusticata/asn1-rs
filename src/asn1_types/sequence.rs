use crate::traits::*;
use crate::{Any, Error, ParseResult, Result, Tag};
use std::borrow::Cow;
use std::convert::TryFrom;

mod iterator;
mod sequence_of;
mod vec;

pub use iterator::*;
pub use sequence_of::*;
pub use vec::*;

#[derive(Clone, Debug)]
pub struct Sequence<'a> {
    pub content: Cow<'a, [u8]>,
}

impl<'a> Sequence<'a> {
    pub const fn new(content: Cow<'a, [u8]>) -> Self {
        Sequence { content }
    }

    pub fn parse<F, T>(&'a self, mut f: F) -> ParseResult<'a, T>
    where
        F: FnMut(&'a [u8]) -> ParseResult<'a, T>,
    {
        let input: &[u8] = &self.content;
        f(input)
    }

    pub fn parse_ref<F, T>(self, mut f: F) -> ParseResult<'a, T>
    where
        F: FnMut(&'a [u8]) -> ParseResult<'a, T>,
    {
        match self.content {
            Cow::Borrowed(b) => f(b),
            _ => Err(nom::Err::Failure(Error::LifetimeError)),
        }
    }

    pub fn ber_iter<T>(&'a self) -> SequenceIterator<'a, T, BerParser>
    where
        T: FromBer<'a>,
    {
        SequenceIterator::new(&self.content)
    }

    pub fn der_iter<T>(&'a self) -> SequenceIterator<'a, T, DerParser>
    where
        T: FromDer<'a>,
    {
        SequenceIterator::new(&self.content)
    }

    pub fn ber_sequence_of<T>(&'a self) -> Result<Vec<T>>
    where
        T: FromBer<'a>,
    {
        self.ber_iter().collect()
    }

    pub fn der_sequence_of<T>(&'a self) -> Result<Vec<T>>
    where
        T: FromDer<'a>,
    {
        self.der_iter().collect()
    }

    pub fn into_ber_sequence_of<T, U>(self) -> Result<Vec<T>>
    where
        for<'b> T: FromBer<'b>,
        T: ToStatic<Owned = T>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SequenceIterator::<T, BerParser>::new(bytes).collect(),
            Cow::Owned(data) => {
                let v1 =
                    SequenceIterator::<T, BerParser>::new(&data).collect::<Result<Vec<T>>>()?;
                let v2 = v1.iter().map(|t| t.to_static()).collect::<Vec<_>>();
                Ok(v2)
            }
        }
    }

    pub fn into_der_sequence_of<T, U>(self) -> Result<Vec<T>>
    where
        for<'b> T: FromDer<'b>,
        T: ToStatic<Owned = T>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SequenceIterator::<T, DerParser>::new(bytes).collect(),
            Cow::Owned(data) => {
                let v1 =
                    SequenceIterator::<T, DerParser>::new(&data).collect::<Result<Vec<T>>>()?;
                let v2 = v1.iter().map(|t| t.to_static()).collect::<Vec<_>>();
                Ok(v2)
            }
        }
    }
}

impl<'a> ToStatic for Sequence<'a> {
    type Owned = Sequence<'static>;

    fn to_static(&self) -> Self::Owned {
        Sequence {
            content: Cow::Owned(self.content.to_vec()),
        }
    }
}

impl<'a, T, U> ToStatic for Vec<T>
where
    T: ToStatic<Owned = U>,
    U: 'static,
{
    type Owned = Vec<U>;

    fn to_static(&self) -> Self::Owned {
        self.iter().map(|t| t.to_static()).collect()
    }
}

impl<'a> AsRef<[u8]> for Sequence<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.content
    }
}

impl<'a> TryFrom<Any<'a>> for Sequence<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Sequence<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        Ok(Sequence {
            content: any.into_cow(),
        })
    }
}

impl<'a> CheckDerConstraints for Sequence<'a> {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl<'a> Tagged for Sequence<'a> {
    const TAG: Tag = Tag::Sequence;
}
