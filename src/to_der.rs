#![cfg(feature = "std")]

use std::io::Write;

use crate::to_ber::*;
use crate::Class;
use crate::DynTagged;
use crate::Length;
use crate::SerializeResult;
use crate::Tag;

/// Common trait for DER encoding functions
///
/// The `Encoder` type allows specifying common encoders for objects with similar headers
/// (for ex. primitive objects) easily.
///
/// # Examples
///
/// Objects from this crate can be encoded as DER:
///
/// ```
/// use asn1_rs::{Integer, ToDer};
///
/// let int = Integer::from(4u32);
/// let mut writer = Vec::new();
/// let sz = int.der_encode(&mut writer).expect("serialization failed");
///
/// assert_eq!(&writer, &[0x02, 0x01, 0x04]);
/// # assert_eq!(sz, 3);
/// ```
///
/// Many of the primitive types can also directly be encoded as DER:
///
/// ```
/// use asn1_rs::ToDer;
///
/// let mut writer = Vec::new();
/// let sz = 4.der_encode(&mut writer).expect("serialization failed");
///
/// assert_eq!(&writer, &[0x02, 0x01, 0x04]);
/// # assert_eq!(sz, 3);
/// ```
pub trait ToDer {
    type Encoder: BerEncoder;

    /// Returns the length of the encoded content of the object
    ///
    /// The length describes the _content_ only, not the header.
    fn der_content_len(&self) -> Length;

    /// Returns the total length (including header) of the encoded content of the object
    fn der_total_len(&self) -> Length {
        let (_, _, tag) = self.der_tag_info();
        let content_length = self.der_content_len();
        ber_total_length(tag, content_length)
    }

    /// Return the tag information to be encoded in header
    fn der_tag_info(&self) -> (Class, bool, Tag);

    /// Encode and write the content of the object to the writer `target`
    ///
    /// Returns the number of bytes written
    fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize>;

    /// Encode and write the header of the object to the writer `target`
    ///
    /// Returns the number of bytes written
    fn der_write_header<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
        let mut encoder = Self::Encoder::new();

        let mut sz = 0;
        let (class, constructed, tag) = self.der_tag_info();
        sz += encoder.write_tag_info(class, constructed, tag, target)?;

        // write length
        let length = self.der_content_len();
        sz += encoder.write_length(length, target)?;

        Ok(sz)
    }

    /// Encode and write the object (header + content) to the writer `target`
    ///
    /// Returns the number of bytes written
    fn der_encode<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
        let sz = self.der_write_header(target)? + self.der_write_content(target)?;

        Ok(sz)
    }

    /// Write the DER encoded representation to a newly allocated `Vec<u8>`
    fn to_der_vec(&self) -> SerializeResult<Vec<u8>> {
        let mut v = Vec::new();
        self.der_encode(&mut v)?;
        Ok(v)
    }

    /// Encode in DER and write the object (header + content) to the writer `target`
    ///
    /// Returns the number of bytes written
    fn write_der<W: Write>(&self, writer: &mut W) -> SerializeResult<usize> {
        self.der_encode(writer)
    }

    //--- DEPRECATED

    /// Get the length of the object (including the header), when encoded
    ///
    // Since we are using DER, length cannot be Indefinite
    #[deprecated(since = "0.8.0", note = "Use `der_total_len()` instead.")]
    fn to_der_len(&self) -> crate::Result<usize> {
        match self.der_content_len() {
            Length::Definite(sz) => Ok(sz),
            Length::Indefinite => Err(crate::Error::IndefiniteLengthUnexpected),
        }
    }

    #[deprecated(since = "0.8.0", note = "Use `to_der_vec()` instead.")]
    fn to_der_vec_raw(&self) -> SerializeResult<Vec<u8>> {
        self.to_der_vec()
    }
}

//--- blanket impls

impl<E, T: ToDer> ToDer for &'_ T
where
    T: ToDer<Encoder = E>,
    E: BerEncoder,
{
    type Encoder = <T as ToDer>::Encoder;

    fn der_content_len(&self) -> Length {
        (*self).der_content_len()
    }

    fn der_tag_info(&self) -> (Class, bool, Tag) {
        (*self).der_tag_info()
    }

    fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
        (*self).der_write_content(target)
    }
}

//--- Macros

/// Helper macro to implement [`ToDer`] for types where implementation is the same as [`ToBer`]
#[macro_export]
macro_rules! impl_toder_from_tober {
    (TY $ty:ty) => {
        impl $crate::ToDer for $ty
        where
            $ty: ToBer,
        {
            type Encoder = <$ty as ToBer>::Encoder;

            fn der_content_len(&self) -> Length {
                <$ty as ToBer>::ber_content_len(self)
            }

            fn der_tag_info(&self) -> (Class, bool, Tag) {
                <$ty as ToBer>::ber_tag_info(self)
            }

            fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
                <$ty as ToBer>::ber_write_content(self, target)
            }
        }
    };
    (LFT $lft:lifetime, $ty:ty) => {
        impl<$lft> $crate::ToDer for $ty
        where
            $ty: ToBer,
        {
            type Encoder = <$ty as ToBer>::Encoder;

            fn der_content_len(&self) -> Length {
                <$ty as ToBer>::ber_content_len(self)
            }

            fn der_tag_info(&self) -> (Class, bool, Tag) {
                <$ty as ToBer>::ber_tag_info(self)
            }

            fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
                <$ty as ToBer>::ber_write_content(self, target)
            }
        }
    };
}

//--- Helper functions

/// Return the length (in bytes) required for a set of objects (DER)
///
/// Compute the length by iterating through all items, and add lengths for their header+content.
///
/// Note: if one of the objects has an undefinite length, then the resulting length
/// will be indefinite.
pub fn der_length_constructed_items<'a, T, IT>(iter: IT) -> Length
where
    T: ToDer + DynTagged + 'a,
    IT: Iterator<Item = &'a T>,
{
    iter.map(|t| t.der_total_len()).sum()
}
