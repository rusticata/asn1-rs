use crate::{
    Any, BerParser, DerParser, FromBer, FromDer, ParseResult, Result, Set, SetIterator, Tag, Tagged,
};
use std::{
    borrow::Cow,
    collections::{BTreeSet, HashSet},
    hash::Hash,
};

#[derive(Debug)]
pub struct SetOf<T> {
    items: Vec<T>,
}

impl<T> SetOf<T> {
    pub const fn new(items: Vec<T>) -> Self {
        SetOf { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<'a, T> AsRef<[T]> for SetOf<T> {
    fn as_ref(&self) -> &[T] {
        &self.items
    }
}

impl<'a, T> FromBer<'a> for SetOf<T>
where
    T: FromBer<'a>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, set) = Set::from_ber(bytes)?;
        let data = match set.content {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SetIterator::<T, BerParser>::new(data).collect::<Result<Vec<T>>>()?;
        Ok((rem, SetOf::new(v)))
    }
}

impl<'a, T> FromDer<'a> for SetOf<T>
where
    T: FromDer<'a>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, set) = Set::from_der(bytes)?;
        let data = match set.content {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SetIterator::<T, DerParser>::new(data).collect::<Result<Vec<T>>>()?;
        Ok((rem, SetOf::new(v)))
    }
}

impl<T> From<SetOf<T>> for Vec<T> {
    fn from(set: SetOf<T>) -> Self {
        set.items
    }
}

impl<T> Tagged for SetOf<T> {
    const TAG: Tag = Tag::Set;
}

impl<T> Tagged for BTreeSet<T> {
    const TAG: Tag = Tag::Set;
}

impl<'a, T> FromBer<'a> for BTreeSet<T>
where
    T: FromBer<'a>,
    T: Ord,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        any.header.assert_tag(Self::TAG)?;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SetIterator::<T, BerParser>::new(data).collect::<Result<BTreeSet<T>>>()?;
        Ok((rem, v))
    }
}

impl<'a, T> FromDer<'a> for BTreeSet<T>
where
    T: FromDer<'a>,
    T: Ord,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<Self> {
        let (rem, any) = Any::from_der(bytes)?;
        any.header.assert_tag(Self::TAG)?;
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            _ => unreachable!(),
        };
        let v = SetIterator::<T, DerParser>::new(data).collect::<Result<BTreeSet<T>>>()?;
        Ok((rem, v))
    }
}

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
