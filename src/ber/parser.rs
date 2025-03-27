use crate::error::*;
use crate::header::*;
use crate::{BerMode, BerParser, DerMode, Input, Length, Tag};
use nom::bytes::streaming::take;
use nom::{Err, IResult, Input as _};

/// Default maximum recursion limit
pub const MAX_RECURSION: usize = 50;

// /// Default maximum object size (2^32)
// pub const MAX_OBJECT_SIZE: usize = 4_294_967_295;

pub trait GetObjectContent {
    /// Return the raw content (bytes) of the next ASN.1 encoded object
    ///
    /// Note: if using BER and length is indefinite, terminating End-Of-Content is NOT included
    fn get_object_content<'a>(
        hdr: &'_ Header,
        i: Input<'a>,
        max_depth: usize,
    ) -> IResult<Input<'a>, Input<'a>, BerError<Input<'a>>>;
}

impl GetObjectContent for BerMode {
    fn get_object_content<'a>(
        hdr: &'_ Header,
        i: Input<'a>,
        max_depth: usize,
    ) -> IResult<Input<'a>, Input<'a>, BerError<Input<'a>>> {
        let start_i = i.clone();
        let (i, _) = ber_skip_object_content(i, hdr, max_depth)?;
        let len = i.span().start - start_i.span().start;
        if hdr.length().is_definite() {
            Ok(start_i.take_split(len))
        } else {
            assert!(len >= 2);
            // take content (minus EndOfContent) and return i (after EndOfContent)
            let content = start_i.take(len - 2);
            Ok((i, content))
        }
    }
}

/// Read the content bytes matching length defined in `header` (BER)
///
/// This function is an alias to [`BerMode::get_object_content`],
/// with default parameters (including recursing limit)
pub fn ber_get_content<'i>(
    header: &Header,
    input: Input<'i>,
) -> IResult<Input<'i>, Input<'i>, BerError<Input<'i>>> {
    BerMode::get_object_content(header, input, MAX_RECURSION)
}

impl GetObjectContent for DerMode {
    /// Skip object content, accepting only DER
    ///
    /// This this function is for DER only, it cannot go into recursion (no indefinite length)
    fn get_object_content<'a>(
        hdr: &'_ Header,
        i: Input<'a>,
        _max_depth: usize,
    ) -> IResult<Input<'a>, Input<'a>, BerError<Input<'a>>> {
        match hdr.length {
            Length::Definite(l) => take(l)(i),
            Length::Indefinite => Err(Err::Error(BerError::new(
                i,
                InnerError::DerConstraintFailed(DerConstraint::IndefiniteLength),
            ))),
        }
    }
}

/// Read the content bytes matching length defined in `header` (BER)
///
/// This function is an alias to [`DerMode::get_object_content`],
/// with default parameters.
pub fn der_get_content<'i>(
    header: &Header,
    input: Input<'i>,
) -> IResult<Input<'i>, Input<'i>, BerError<Input<'i>>> {
    DerMode::get_object_content(header, input, MAX_RECURSION)
}

/// Skip object content, and return true if object was End-Of-Content
fn ber_skip_object_content<'a>(
    i: Input<'a>,
    hdr: &Header,
    max_depth: usize,
) -> IResult<Input<'a>, bool, BerError<Input<'a>>> {
    if max_depth == 0 {
        return Err(Err::Error(BerError::new(i, InnerError::BerMaxDepth)));
    }
    match hdr.length {
        Length::Definite(l) => {
            if l == 0 && hdr.tag == Tag::EndOfContent {
                return Ok((i, true));
            }
            let (i, _) = take(l)(i)?;
            Ok((i, false))
        }
        Length::Indefinite => {
            hdr.assert_constructed_inner()
                .map_err(BerError::convert(i.clone()))?;
            // read objects until EndOfContent (00 00)
            // this is recursive
            let mut i = i;
            loop {
                let (i2, header2) = Header::parse_ber(i)?;
                let (i3, eoc) = ber_skip_object_content(i2, &header2, max_depth - 1)?;
                if eoc {
                    // return false, since top object was not EndOfContent
                    return Ok((i3, false));
                }
                i = i3;
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! der_constraint_fail_if(
    ($slice:expr, $cond:expr, $constraint:expr) => (
        {
            if $cond {
                return Err(::nom::Err::Error(Error::DerConstraintFailed($constraint)));
            }
        }
    );
);
