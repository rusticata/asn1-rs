use crate::traits::*;
use crate::{Any, Error, ParseResult, Result, Tag};
use std::borrow::Cow;
use std::convert::TryFrom;

mod btreeset;
mod hashset;
mod iterator;
mod set_of;

pub use btreeset::*;
pub use hashset::*;
pub use iterator::*;
pub use set_of::*;

#[derive(Clone, Debug)]
pub struct Set<'a> {
    pub content: Cow<'a, [u8]>,
}

impl<'a> Set<'a> {
    pub const fn new(content: Cow<'a, [u8]>) -> Self {
        Set { content }
    }

    pub fn parse<F, T>(&'a self, mut f: F) -> ParseResult<'a, T>
    where
        F: FnMut(&'a [u8]) -> ParseResult<'a, T>,
    {
        let input: &[u8] = &self.content;
        f(input)
    }

    pub fn ber_iter<T>(&'a self) -> SetIterator<'a, T, BerParser>
    where
        T: FromBer<'a>,
    {
        SetIterator::new(&self.content)
    }

    pub fn der_iter<T>(&'a self) -> SetIterator<'a, T, DerParser>
    where
        T: FromDer<'a>,
    {
        SetIterator::new(&self.content)
    }

    pub fn ber_set_of<T>(&'a self) -> Result<Vec<T>>
    where
        T: FromBer<'a>,
    {
        self.ber_iter().collect()
    }

    pub fn der_set_of<T>(&'a self) -> Result<Vec<T>>
    where
        T: FromDer<'a>,
    {
        self.der_iter().collect()
    }

    pub fn into_ber_set_of<T>(self) -> Result<Vec<T>>
    where
        for<'b> T: FromBer<'b>,
        T: ToStatic<Owned = T>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SetIterator::<T, BerParser>::new(bytes).collect(),
            Cow::Owned(data) => {
                let v1 = SetIterator::<T, BerParser>::new(&data).collect::<Result<Vec<T>>>()?;
                let v2 = v1.iter().map(|t| t.to_static()).collect::<Vec<_>>();
                Ok(v2)
            }
        }
    }

    pub fn into_der_set_of<T>(self) -> Result<Vec<T>>
    where
        for<'b> T: FromDer<'b>,
        T: ToStatic<Owned = T>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SetIterator::<T, DerParser>::new(bytes).collect(),
            Cow::Owned(data) => {
                let v1 = SetIterator::<T, DerParser>::new(&data).collect::<Result<Vec<T>>>()?;
                let v2 = v1.iter().map(|t| t.to_static()).collect::<Vec<_>>();
                Ok(v2)
            }
        }
    }

    pub fn into_der_set_of_ref<T>(self) -> Result<Vec<T>>
    where
        T: FromDer<'a>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SetIterator::<T, DerParser>::new(bytes).collect(),
            Cow::Owned(_) => Err(Error::LifetimeError),
        }
    }
}

impl<'a> ToStatic for Set<'a> {
    type Owned = Set<'static>;

    fn to_static(&self) -> Self::Owned {
        Set {
            content: Cow::Owned(self.content.to_vec()),
        }
    }
}

impl<'a> AsRef<[u8]> for Set<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.content
    }
}

impl<'a> TryFrom<Any<'a>> for Set<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Set<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        Ok(Set {
            content: any.into_cow(),
        })
    }
}

impl<'a> CheckDerConstraints for Set<'a> {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl<'a> Tagged for Set<'a> {
    const TAG: Tag = Tag::Set;
}
