use crate::error::*;
use crate::header::*;
use crate::{BerMode, BerParser, DerMode, Input, Length, Tag};
use nom::bytes::streaming::take;
use nom::{Err, IResult, Input as _, Needed};
use rusticata_macros::custom_check;

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

/// Try to parse input bytes as u64
#[inline]
pub(crate) fn bytes_to_u64(s: &[u8]) -> core::result::Result<u64, Error> {
    let mut u: u64 = 0;
    for &c in s {
        if u & 0xff00_0000_0000_0000 != 0 {
            return Err(Error::IntegerTooLarge);
        }
        u <<= 8;
        u |= u64::from(c);
    }
    Ok(u)
}

pub(crate) fn parse_identifier(i: &[u8]) -> ParseResult<(u8, u8, u32, &[u8])> {
    if i.is_empty() {
        Err(Err::Incomplete(Needed::new(1)))
    } else {
        let a = i[0] >> 6;
        let b = u8::from(i[0] & 0b0010_0000 != 0);
        let mut c = u32::from(i[0] & 0b0001_1111);

        let mut tag_byte_count = 1;

        if c == 0x1f {
            c = 0;
            loop {
                // Make sure we don't read past the end of our data.
                custom_check!(i, tag_byte_count >= i.len(), Error::InvalidTag)?;

                // With tag defined as u32 the most we can fit in is four tag bytes.
                // (X.690 doesn't actually specify maximum tag width.)
                custom_check!(i, tag_byte_count > 5, Error::InvalidTag)?;

                c = (c << 7) | (u32::from(i[tag_byte_count]) & 0x7f);
                let done = i[tag_byte_count] & 0x80 == 0;
                tag_byte_count += 1;
                if done {
                    break;
                }
            }
        }

        let (raw_tag, rem) = i.split_at(tag_byte_count);

        Ok((rem, (a, b, c, raw_tag)))
    }
}

/// Return the MSB and the rest of the first byte, or an error
pub(crate) fn parse_ber_length_byte(i: &[u8]) -> ParseResult<(u8, u8)> {
    if i.is_empty() {
        Err(Err::Incomplete(Needed::new(1)))
    } else {
        let a = i[0] >> 7;
        let b = i[0] & 0b0111_1111;
        Ok((&i[1..], (a, b)))
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
