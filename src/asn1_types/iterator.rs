use core::iter::FromIterator;
use core::marker::PhantomData;

use nom::{Err, IResult, Input as _};

use crate::{ASN1Mode, BerError, BerMode, BerParser, DerMode, DerParser, InnerError, Input};

use super::Any;

/// Iterator for constructed sub-objects contained in an ANY object
///
/// In ASN.1, this is supposed to happen only for constructed objects, but this iterator
/// does not test if `Self` is constructed.
///
/// Each call to `.next()` returns one of the following:
/// - `Some(Ok((input, obj)))` if an object was parsed.\
///   `obj` is the next sub-object, and has type [`Any`].\
///   `input` is the span of the sub-object.\
///   Unless caller wants information of current offset, `input` can be ignored
/// - `Some(Err(e))` if an error occurred.\
///   Note: If `e` is `Incomplete`, this means an object has as start but is missing bytes
/// - `None`: no more objects
///
/// Note: to avoid infinite loops, calling `.next()` after an error happened will
/// return `None` (ending iteration).
#[derive(Debug)]
pub struct AnyIterator<'a, Mode>
where
    Mode: ASN1Mode,
{
    input: Input<'a>,
    has_error: bool,
    _mode: PhantomData<*const Mode>,
}

impl<'a, Mode> AnyIterator<'a, Mode>
where
    Mode: ASN1Mode,
{
    pub fn new(input: Input<'a>) -> Self {
        Self {
            input,
            has_error: false,
            _mode: PhantomData,
        }
    }
}

impl<'a> AnyIterator<'a, BerMode> {
    /// Try to iterate on sub-objects, returning a collection `B` of elements with type `T`
    ///
    /// Similarly to [`Iterator::collect`], this function requires type annotations.
    ///
    /// Since it also has to infer the error type, it is often not possible to use the `?`
    /// operator directly: `.try_parse_collect()?`will cause an error because it cannot infer
    /// result type. To avoid this, use an itermediate value to store result (or use the
    /// turbofish `::<>` operator).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use asn1_rs::{AnyIterator, BerMode, Input};
    /// use hex_literal::hex;
    ///
    /// let input = Input::from_slice(&hex!("0203010001 0203010001"));
    /// let mut iter = AnyIterator::<BerMode>::new(input);
    ///
    /// let r: Result<(Input, Vec<u32>), _> = iter.try_parse_collect();
    /// let _ = r.expect("parsing failed");
    ///
    /// let r = iter.try_parse_collect::<Vec<u32>, _>();
    /// let _ = r.expect("parsing failed");
    /// ```
    pub fn try_parse_collect<B, T>(&mut self) -> IResult<Input<'a>, B, <T as BerParser<'a>>::Error>
    where
        B: FromIterator<T>,
        T: BerParser<'a>,
        <T as BerParser<'a>>::Error: From<BerError<Input<'a>>>,
    {
        let b = <Result<B, Err<T::Error, T::Error>>>::from_iter(self.map(|r| match r {
            Ok((_, obj)) => {
                let (_, obj) = T::from_ber_content(&obj.header, obj.data)?;
                Ok(obj)
            }
            Err(e) => Err(Err::Error(e.into())),
        }));
        // after iteration, self.input points at end of last object content
        b.map(|obj| (self.input.clone(), obj))
    }
}

impl<'a> AnyIterator<'a, DerMode> {
    /// Try to iterate on sub-objects, returning a collection `B` of elements with type `T`
    ///
    /// Similarly to [`Iterator::collect`], this function requires type annotations.
    ///
    /// Since it also has to infer the error type, it is often not possible to use the `?`
    /// operator directly: `.try_parse_collect()?`will cause an error because it cannot infer
    /// result type. To avoid this, use an itermediate value to store result (or use the
    /// turbofish `::<>` operator).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use asn1_rs::{AnyIterator, DerMode, Input};
    /// use hex_literal::hex;
    ///
    /// let input = Input::from_slice(&hex!("0203010001 0203010001"));
    /// let mut iter = AnyIterator::<DerMode>::new(input);
    ///
    /// let r: Result<(Input, Vec<u32>), _> = iter.try_parse_collect();
    /// let _ = r.expect("parsing failed");
    ///
    /// let r = iter.try_parse_collect::<Vec<u32>, _>();
    /// let _ = r.expect("parsing failed");
    /// ```
    pub fn try_parse_collect<B, T>(&mut self) -> IResult<Input<'a>, B, <T as DerParser<'a>>::Error>
    where
        B: FromIterator<T>,
        T: DerParser<'a>,
        <T as DerParser<'a>>::Error: From<BerError<Input<'a>>>,
    {
        let b = <Result<B, Err<T::Error, T::Error>>>::from_iter(self.map(|r| match r {
            Ok((_, obj)) => {
                let (_, obj) = T::from_der_content(&obj.header, obj.data)?;
                Ok(obj)
            }
            Err(e) => Err(Err::Error(e.into())),
        }));
        // after iteration, self.input points at end of last object content
        b.map(|obj| (self.input.clone(), obj))
    }
}

impl<'a> Iterator for AnyIterator<'a, BerMode> {
    type Item = Result<(Input<'a>, Any<'a>), BerError<Input<'a>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error || self.input.is_empty() {
            return None;
        }
        let input = self.input.clone();
        match Any::parse_ber(input) {
            Ok((rem, obj)) => {
                let data = self.input.take(rem.span().start - self.input.span().start);
                self.input = rem;
                Some(Ok((data, obj)))
            }
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                self.has_error = true;
                Some(Err(e))
            }
            Err(Err::Incomplete(n)) => {
                self.has_error = true;
                let e = BerError::new(self.input.clone(), InnerError::Incomplete(n));
                Some(Err(e))
            }
        }
    }
}

impl<'a> Iterator for AnyIterator<'a, DerMode> {
    type Item = Result<(Input<'a>, Any<'a>), BerError<Input<'a>>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.has_error || self.input.is_empty() {
            return None;
        }
        let input = self.input.clone();
        match Any::parse_der(input) {
            Ok((rem, obj)) => {
                let data = self.input.take(rem.span().start - self.input.span().start);
                self.input = rem;
                Some(Ok((data, obj)))
            }
            Err(Err::Error(e)) | Err(Err::Failure(e)) => {
                self.has_error = true;
                Some(Err(e))
            }
            Err(Err::Incomplete(n)) => {
                self.has_error = true;
                let e = BerError::new(self.input.clone(), InnerError::Incomplete(n));
                Some(Err(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use hex_literal::hex;

    use crate::{Any, BerMode, BerParser, DerMode, DerParser, Input, Tag};

    use super::AnyIterator;

    #[test]
    fn any_iterator_ber() {
        // Ok: empty
        let input = Input::from_slice(&hex!("30 00"));
        let (_, any) = Any::parse_ber(input).expect("Parsing failed");
        let iter = any.iter_elements::<BerMode>();
        assert_eq!(iter.count(), 0);

        // Ok: 1 item
        let input = Input::from_slice(&hex!("30 05 0203010001"));
        let (_, any) = Any::parse_ber(input).expect("Parsing failed");
        let mut iter = any.iter_elements::<BerMode>();
        let (r, obj0) = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(r.span(), &(2..7));
        assert_eq!(obj0.tag(), Tag::Integer);
        assert_eq!(iter.next(), None);

        // Ok: 2 items with different tags
        let input = Input::from_slice(&hex!("30 08 02 03 01 00 01 01 01 ff"));
        let (_, any) = Any::parse_ber(input).expect("Parsing failed");
        let tags = any
            .iter_elements::<BerMode>()
            .map(|r| r.expect("parsing sub-object failed").1.tag())
            .collect::<Vec<_>>();
        assert_eq!(&tags, &[Tag::Integer, Tag::Boolean]);

        // Fail: incomplete
        let input = Input::from_slice(&hex!("30 07 0203010001 0201"));
        let (_, any) = Any::parse_ber(input).expect("Parsing failed");
        let mut iter = any.iter_elements::<BerMode>();
        let _ = iter.next().expect("empty iter").expect("subject-object 0");
        let _ = iter
            .next()
            .expect("empty iter")
            .expect_err("subject-object 1 should fail");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn any_iterator_der() {
        // Ok: empty
        let input = Input::from_slice(&hex!("30 00"));
        let (_, any) = Any::parse_der(input).expect("Parsing failed");
        let iter = any.iter_elements::<DerMode>();
        assert_eq!(iter.count(), 0);

        // Ok: 1 item
        let input = Input::from_slice(&hex!("30 05 0203010001"));
        let (_, any) = Any::parse_der(input).expect("Parsing failed");
        let mut iter = any.iter_elements::<DerMode>();
        let (r, obj0) = iter.next().expect("empty iter").expect("subject-object 0");
        assert_eq!(r.span(), &(2..7));
        assert_eq!(obj0.tag(), Tag::Integer);
        assert_eq!(iter.next(), None);

        // Ok: 2 items with different tags
        let input = Input::from_slice(&hex!("30 08 02 03 01 00 01 01 01 ff"));
        let (_, any) = Any::parse_der(input).expect("Parsing failed");
        let tags = any
            .iter_elements::<DerMode>()
            .map(|r| r.expect("parsing sub-object failed").1.tag())
            .collect::<Vec<_>>();
        assert_eq!(&tags, &[Tag::Integer, Tag::Boolean]);

        // Fail: incomplete
        let input = Input::from_slice(&hex!("30 07 0203010001 0201"));
        let (_, any) = Any::parse_der(input).expect("Parsing failed");
        let mut iter = any.iter_elements::<DerMode>();
        let _ = iter.next().expect("empty iter").expect("subject-object 0");
        let _ = iter
            .next()
            .expect("empty iter")
            .expect_err("subject-object 1 should fail");
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn any_iterator_try_collect() {
        let input = Input::from_slice(&hex!("0203010001 0203010000"));
        let mut iter = AnyIterator::<BerMode>::new(input);

        // let r = iter.try_parse_collect::<Vec<_>, u32>();
        // let r: Result<(Input, Vec<u32>), _> = iter.try_parse_collect();
        let (rem, v) = iter.try_parse_collect::<Vec<u32>, _>().unwrap();
        assert!(rem.is_empty());
        assert_eq!(&v, &[65537, 65536]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn any_iterator_try_collect_std() {
        use std::collections::HashSet;
        let input = Input::from_slice(&hex!("0203010001 0203010000"));
        let mut iter = AnyIterator::<BerMode>::new(input);
        let (rem, h) = iter.try_parse_collect::<HashSet<u32>, _>().unwrap();
        assert!(rem.is_empty());
        assert_eq!(h.len(), 2);
        assert!(h.contains(&65536));
        assert!(h.contains(&65537));
    }
}
