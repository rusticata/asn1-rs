use crate::traits::*;
use crate::{Any, Error, ParseResult, Result, Tag};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::marker::PhantomData;

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
    fn check_constraints(any: &Any) -> Result<()> {
        if !any.header.length.is_definite() {
            return Err(Error::IndefiniteLengthUnexpected);
        }
        Ok(())
    }
}

impl<'a> Tagged for Sequence<'a> {
    const TAG: Tag = Tag::Sequence;
}

#[derive(Debug)]
pub struct SequenceIterator<'a, T, F>
where
    F: ASN1Parser,
{
    data: &'a [u8],
    has_error: bool,
    _t: PhantomData<T>,
    _f: PhantomData<F>,
}

impl<'a, T, F> SequenceIterator<'a, T, F>
where
    F: ASN1Parser,
{
    pub fn new(data: &'a [u8]) -> Self {
        SequenceIterator {
            data,
            has_error: false,
            _t: PhantomData,
            _f: PhantomData,
        }
    }
}

impl<'a, T> Iterator for SequenceIterator<'a, T, BerParser>
where
    T: FromBer<'a>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error || self.data.is_empty() {
            return None;
        }
        match T::from_ber(&self.data) {
            Ok((rem, obj)) => {
                self.data = rem;
                Some(Ok(obj))
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                self.has_error = true;
                Some(Err(e))
            }

            Err(nom::Err::Incomplete(n)) => {
                self.has_error = true;
                Some(Err(Error::Incomplete(n)))
            }
        }
    }
}

impl<'a, T> Iterator for SequenceIterator<'a, T, DerParser>
where
    T: FromDer<'a>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error || self.data.is_empty() {
            return None;
        }
        match T::from_der(&self.data) {
            Ok((rem, obj)) => {
                self.data = rem;
                Some(Ok(obj))
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                self.has_error = true;
                Some(Err(e))
            }

            Err(nom::Err::Incomplete(n)) => {
                self.has_error = true;
                Some(Err(Error::Incomplete(n)))
            }
        }
    }
}
