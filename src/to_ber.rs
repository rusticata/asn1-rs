#![cfg(feature = "std")]

use std::io::Write;

use crate::{Class, InnerError, Length, SerializeResult, Tag};

mod constructed;
mod constructed_indefinite;
mod encoder;
mod generic;
mod primitive;

pub use constructed::*;
pub use constructed_indefinite::*;
pub use encoder::*;
pub use generic::*;
pub use primitive::*;

/// Common trait for BER encoding functions
///
/// The `Encoder` type allows specifying common encoders for objects with similar headers
/// (for ex. primitive objects) easily.
pub trait ToBer {
    type Encoder: BerEncoder;

    /// Returns the length of the encoded content of the object
    ///
    /// The length describes the _content_ only, not the header.
    fn ber_content_len(&self) -> Length;

    /// Returns the total length (including header) of the encoded content of the object
    fn ber_total_len(&self) -> Length {
        let (_, _, tag) = self.ber_tag_info();
        let content_length = self.ber_content_len();
        ber_total_length(tag, content_length)
    }

    /// Return the tag information to be encoded in header
    fn ber_tag_info(&self) -> (Class, bool, Tag);

    /// Encode and write the content of the object to the writer `target`
    ///
    /// Returns the number of bytes written
    fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize>;

    /// Encode and write the header of the object to the writer `target`
    ///
    /// Returns the number of bytes written
    fn ber_write_header<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
        let mut encoder = Self::Encoder::new();

        let mut sz = 0;
        let (class, constructed, tag) = self.ber_tag_info();
        sz += encoder.write_tag_info(class, constructed, tag, target)?;

        // write length
        let length = self.ber_content_len();
        sz += encoder.write_length(length, target)?;

        Ok(sz)
    }

    /// Encode and write the object (header + content) to the writer `target`
    ///
    /// Returns the number of bytes written
    fn ber_encode<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
        let sz = self.ber_write_header(target)? + self.ber_write_content(target)?;

        Ok(sz)
    }

    /// Write the DER encoded representation to a newly allocated `Vec<u8>`
    fn to_ber_vec(&self) -> SerializeResult<Vec<u8>> {
        let mut v = Vec::new();
        self.ber_encode(&mut v)?;
        Ok(v)
    }

    /// Encode in BER and write the object (header + content) to the writer `target`
    ///
    /// Returns the number of bytes written
    fn write_ber<W: Write>(&self, writer: &mut W) -> SerializeResult<usize> {
        self.ber_encode(writer)
    }
}

//--- blanket impls

impl<E, T: ToBer> ToBer for &'_ T
where
    T: ToBer<Encoder = E>,
    E: BerEncoder,
{
    type Encoder = <T as ToBer>::Encoder;

    fn ber_content_len(&self) -> Length {
        (*self).ber_content_len()
    }

    fn ber_tag_info(&self) -> (Class, bool, Tag) {
        (*self).ber_tag_info()
    }

    fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
        (*self).ber_write_content(target)
    }
}

//--- helper functions

/// Returns the length (in bytes) required for the given tag
pub fn ber_tag_length(tag: Tag) -> usize {
    match tag.0 {
        0..=30 => 1,
        t => {
            let mut sz = 1;
            let mut val = t;
            loop {
                if val <= 127 {
                    return sz + 1;
                } else {
                    val >>= 7;
                    sz += 1;
                }
            }
        }
    }
}

/// Returns the length (in bytes) required for the given length
pub fn ber_length_length(length: Length) -> Result<usize, InnerError> {
    match length {
        Length::Indefinite => Ok(1),
        Length::Definite(l) => match l {
            0..=0x7f => Ok(1),
            0x80..=0xff => Ok(2),
            0x100..=0xffff => Ok(3),
            0x1_0000..=0xffff_ffff => Ok(4),
            _ => Err(InnerError::InvalidLength),
        },
    }
}

/// Returns the total length (header+content) required for an object, given the input parameters
///
/// If the content length is `Indefinite`, then the result will also be `Indefinite`
pub fn ber_total_length(tag: Tag, content_length: Length) -> Length {
    let header_len = ber_header_length(tag, content_length).unwrap_or(0);
    content_length + header_len
}

/// Returns the length (in bytes) required for the full header (tag+length)
///
/// Returns 0 if length is invalid (overflow)
pub fn ber_header_length(tag: Tag, length: Length) -> Result<usize, InnerError> {
    let sz = ber_tag_length(tag);
    let sz = sz + ber_length_length(length)?;
    Ok(sz)
}

/// Return the length (in bytes) required for a set of objects (BER)
///
/// Compute the length by iterating through all items, and add lengths for their header+content.
///
/// Note: if one of the objects has an undefinite length, then the resulting length
/// will be indefinite.
pub fn ber_length_constructed_items<'a, T, IT>(iter: IT) -> Length
where
    T: ToBer + 'a,
    IT: Iterator<Item = &'a T>,
{
    iter.map(|t| t.ber_total_len()).sum()
}
