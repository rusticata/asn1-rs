use core::marker::PhantomData;

use nom::{Err, Input as _};

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
}
