use crate::{
    ASN1Mode, AnyIterator, BerError, BerMode, BerParser, DerMode, DerParser, Error, FromBer,
    FromDer, Input, Tagged,
};
use core::marker::PhantomData;

/// An Iterator over binary data, parsing elements of type `T`
///
/// This helps parsing `SEQUENCE OF` items of type `T`. The type of parser
/// (BER/DER) is specified using the generic parameter `F` of this struct.
///
/// Note: the iterator must start on the sequence *contents*, not the sequence itself.
///
/// # Examples
///
/// ```rust
/// use asn1_rs::{DerMode, Integer, SequenceIterator};
///
/// let data = &[0x30, 0x6, 0x2, 0x1, 0x1, 0x2, 0x1, 0x2];
/// for (idx, item) in SequenceIterator::<Integer, DerMode>::new(&data[2..]).enumerate() {
///     let item = item.unwrap(); // parsing could have failed
///     let i = item.as_u32().unwrap(); // integer can be negative, or too large to fit into u32
///     assert_eq!(i as usize, idx + 1);
/// }
/// ```
#[derive(Debug)]
pub struct SequenceIterator<'a, T, F, E = Error>
where
    F: ASN1Mode,
{
    data: &'a [u8],
    has_error: bool,
    _t: PhantomData<T>,
    _f: PhantomData<F>,
    _e: PhantomData<E>,
}

impl<'a, T, F, E> SequenceIterator<'a, T, F, E>
where
    F: ASN1Mode,
{
    pub fn new(data: &'a [u8]) -> Self {
        SequenceIterator {
            data,
            has_error: false,
            _t: PhantomData,
            _f: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<'a, T, E> Iterator for SequenceIterator<'a, T, BerMode, E>
where
    T: FromBer<'a, E>,
    E: From<Error>,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error || self.data.is_empty() {
            return None;
        }
        match T::from_ber(self.data) {
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
                Some(Err(Error::Incomplete(n).into()))
            }
        }
    }
}

impl<'a, T, E> Iterator for SequenceIterator<'a, T, DerMode, E>
where
    T: FromDer<'a, E>,
    E: From<Error>,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error || self.data.is_empty() {
            return None;
        }
        match T::from_der(self.data) {
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
                Some(Err(Error::Incomplete(n).into()))
            }
        }
    }
}

#[derive(Debug)]
pub struct SequenceIteratorInput<'a, T, Mode, E = BerError<Input<'a>>>
where
    Mode: ASN1Mode,
{
    iter: AnyIterator<'a, Mode>,
    t: PhantomData<*const T>,
    e: PhantomData<*const E>,
}

impl<'a, T, E, Mode> SequenceIteratorInput<'a, T, Mode, E>
where
    Mode: ASN1Mode,
{
    pub fn new(input: Input<'a>) -> Self {
        let iter = AnyIterator::new(input);
        Self {
            iter,
            t: PhantomData,
            e: PhantomData,
        }
    }
}

// Build Iterator, by mapping inner iterator
impl<'a, T, E> Iterator for SequenceIteratorInput<'a, T, BerMode, E>
where
    T: BerParser<'a>,
    E: From<<T as BerParser<'a>>::Error> + From<BerError<Input<'a>>>,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok((_, any))) => match T::from_ber_content(any.data, any.header) {
                Ok((_, obj)) => Some(Ok(obj)),
                Err(e) => {
                    let e = match e {
                        nom::Err::Error(e) | nom::Err::Failure(e) => e.into(),
                        nom::Err::Incomplete(n) => {
                            // we need input to build error, but we don't have it
                            let input = Input::from_slice(&[]);
                            BerError::incomplete(input, n).into()
                        }
                    };
                    Some(Err(e))
                }
            },
            Some(Err(e)) => Some(Err(e.into())),
            None => None,
        }
    }
}

// Build Iterator, by mapping inner iterator
impl<'a, T, E> Iterator for SequenceIteratorInput<'a, T, DerMode, E>
where
    T: Tagged,
    T: DerParser<'a>,
    E: From<<T as DerParser<'a>>::Error> + From<BerError<Input<'a>>>,
{
    type Item = Result<T, E>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(Ok((_, any))) => match T::from_der_content(any.data, any.header) {
                Ok((_, obj)) => Some(Ok(obj)),
                Err(e) => {
                    let e = match e {
                        nom::Err::Error(e) | nom::Err::Failure(e) => e.into(),
                        nom::Err::Incomplete(n) => {
                            // we need input to build error, but we don't have it
                            let input = Input::from_slice(&[]);
                            BerError::incomplete(input, n).into()
                        }
                    };
                    Some(Err(e))
                }
            },
            Some(Err(e)) => Some(Err(e.into())),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerMode, DerMode, Input, SequenceIteratorInput};

    #[test]
    fn sequence_iterator_ber() {
        // Ok: empty
        let input = Input::from_slice(&hex!(""));
        let iter = SequenceIteratorInput::<u32, BerMode>::new(input);
        assert_eq!(iter.count(), 0);

        // Ok: 1 item
        let input = Input::from_slice(&hex!("0203010001"));
        let mut iter = SequenceIteratorInput::<u32, BerMode>::new(input);
        let obj0 = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(obj0, 65537);
        assert_eq!(iter.next(), None);

        // Fail: 2 items, 1 with incorrect tag
        let input = Input::from_slice(&hex!("0203010001 0101ff"));
        let mut iter = SequenceIteratorInput::<u32, BerMode>::new(input);
        let obj0 = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(obj0, 65537);
        let _ = iter
            .next()
            .expect("empty iter")
            .expect_err("sub-object 1 should fail");
        assert_eq!(iter.next(), None);

        // Fail: incomplete
        let input = Input::from_slice(&hex!("0203010001 0201"));
        let mut iter = SequenceIteratorInput::<u32, BerMode>::new(input);
        let obj0 = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(obj0, 65537);
        let _ = iter
            .next()
            .expect("empty iter")
            .expect_err("sub-object 1 should fail");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn sequence_iterator_der() {
        // Ok: empty
        let input = Input::from_slice(&hex!(""));
        let iter = SequenceIteratorInput::<u32, DerMode>::new(input);
        assert_eq!(iter.count(), 0);

        // Ok: 1 item
        let input = Input::from_slice(&hex!("0203010001"));
        let mut iter = SequenceIteratorInput::<u32, DerMode>::new(input);
        let obj0 = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(obj0, 65537);
        assert_eq!(iter.next(), None);

        // Fail: 2 items, 1 with incorrect tag
        let input = Input::from_slice(&hex!("0203010001 0101ff"));
        let mut iter = SequenceIteratorInput::<u32, DerMode>::new(input);
        let obj0 = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(obj0, 65537);
        let _ = iter
            .next()
            .expect("empty iter")
            .expect_err("sub-object 1 should fail");
        assert_eq!(iter.next(), None);

        // Fail: incomplete
        let input = Input::from_slice(&hex!("0203010001 0201"));
        let mut iter = SequenceIteratorInput::<u32, DerMode>::new(input);
        let obj0 = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(obj0, 65537);
        let _ = iter
            .next()
            .expect("empty iter")
            .expect_err("sub-object 1 should fail");
        assert_eq!(iter.next(), None);
    }
}
