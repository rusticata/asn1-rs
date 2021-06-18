use crate::{ASN1Parser, BerParser, DerParser, Error, FromBer, FromDer, Result};
use std::marker::PhantomData;

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
