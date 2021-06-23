use crate::error::*;
use crate::traits::*;
use crate::{Any, Tag};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::convert::TryInto;

#[cfg(feature = "bigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
pub use num_bigint::{BigInt, BigUint, Sign};

/// Decode an unsigned integer into a big endian byte slice with all leading
/// zeroes removed.
///
/// Returns a byte array of the requested size containing a big endian integer.
fn decode_slice(any: Any<'_>) -> Result<&[u8]> {
    let bytes = match any.data {
        Cow::Borrowed(b) => b,
        Cow::Owned(_) => unreachable!(),
    };

    // The `INTEGER` type always encodes a signed value, so for unsigned
    // values the leading `0x00` byte may need to be removed.
    //
    // We also disallow a leading byte which would overflow a signed ASN.1
    // integer (since we're decoding an unsigned integer).
    // We expect all such cases to have a leading `0x00` byte.
    //
    // DER check have been moved to CheckDerConstraints
    match bytes {
        [] => Err(Error::DerConstraintFailed),
        [0] => Ok(bytes),
        [0, byte, ..] if *byte < 0x80 => Err(Error::DerConstraintFailed),
        [0, rest @ ..] => Ok(&rest),
        [byte, ..] if *byte >= 0x80 => Err(Error::IntegerTooLarge),
        _ => Ok(bytes),
    }
}

/// Decode an unsigned integer into a byte array of the requested size
/// containing a big endian integer.
fn decode_array_uint<const N: usize>(any: Any<'_>) -> Result<[u8; N]> {
    let input = decode_slice(any)?;

    if input.len() > N {
        return Err(Error::IntegerTooLarge);
    }

    // Input has leading zeroes removed, so we need to add them back
    let mut output = [0u8; N];
    assert!(input.len() <= N);
    output[N.saturating_sub(input.len())..].copy_from_slice(input);
    Ok(output)
}

/// Decode an unsigned integer of the specified size.
///
/// Returns a byte array of the requested size containing a big endian integer.
fn decode_array_int<const N: usize>(any: Any<'_>) -> Result<[u8; N]> {
    if any.data.len() > N {
        return Err(Error::IntegerTooLarge);
    }

    // any.tag().assert_eq(Tag::Integer)?;
    let mut output = [0xFFu8; N];
    let offset = N.saturating_sub(any.as_bytes().len());
    output[offset..].copy_from_slice(any.as_bytes());
    Ok(output)
}

/// Is the highest bit of the first byte in the slice 1? (if present)
#[inline]
fn is_highest_bit_set(bytes: &[u8]) -> bool {
    bytes
        .get(0)
        .map(|byte| byte & 0b10000000 != 0)
        .unwrap_or(false)
}

macro_rules! impl_int {
    ($uint:ty => $int:ty) => {
        impl<'a> TryFrom<Any<'a>> for $int {
            type Error = Error;

            fn try_from(any: Any<'a>) -> Result<Self> {
                any.tag().assert_eq(Self::TAG)?;
                any.header.assert_primitive()?;
                let result = if is_highest_bit_set(any.as_bytes()) {
                    <$uint>::from_be_bytes(decode_array_int(any)?) as $int
                } else {
                    Self::from_be_bytes(decode_array_uint(any)?)
                };
                Ok(result)
            }
        }
        impl<'a> CheckDerConstraints for $int {
            fn check_constraints(any: &Any) -> Result<()> {
                any.header.assert_primitive()?;
                any.header.length.assert_definite()?;
                match any.as_bytes() {
                    [] => Err(Error::DerConstraintFailed),
                    [0] => Ok(()),
                    [0, byte, ..] if *byte < 0x80 => Err(Error::DerConstraintFailed),
                    // [0, ..] => Ok(()),
                    // [byte, ..] if *byte >= 0x80 => Err(Error::IntegerTooLarge),
                    _ => Ok(()),
                }
                // Ok(())
            }
        }

        impl Tagged for $int {
            const TAG: Tag = Tag::Integer;
        }
    };
}

macro_rules! impl_uint {
    ($ty:ty) => {
        impl<'a> TryFrom<Any<'a>> for $ty {
            type Error = Error;

            fn try_from(any: Any<'a>) -> Result<Self> {
                any.tag().assert_eq(Self::TAG)?;
                any.header.assert_primitive()?;
                let result = Self::from_be_bytes(decode_array_uint(any)?);
                Ok(result)
            }
        }
        impl<'a> CheckDerConstraints for $ty {
            fn check_constraints(any: &Any) -> Result<()> {
                any.header.assert_primitive()?;
                any.header.length.assert_definite()?;
                match any.as_bytes() {
                    [] => Err(Error::DerConstraintFailed),
                    [0] => Ok(()),
                    [0, byte, ..] if *byte < 0x80 => Err(Error::DerConstraintFailed),
                    // [0, ..] => Ok(()),
                    // [byte, ..] if *byte >= 0x80 => Err(Error::IntegerTooLarge),
                    _ => Ok(()),
                }
                // Ok(())
            }
        }

        impl Tagged for $ty {
            const TAG: Tag = Tag::Integer;
        }
    };
}

impl_uint!(u8);
impl_uint!(u16);
impl_uint!(u32);
impl_uint!(u64);
impl_uint!(u128);
impl_int!(u8 => i8);
impl_int!(u16 => i16);
impl_int!(u32 => i32);
impl_int!(u64 => i64);
impl_int!(u128 => i128);

#[derive(Debug, PartialEq)]
pub struct Integer<'a> {
    pub(crate) data: Cow<'a, [u8]>,
}

impl<'a> Integer<'a> {
    pub const fn new(s: &'a [u8]) -> Self {
        Integer {
            data: Cow::Borrowed(s),
        }
    }

    pub fn any(&'a self) -> Any<'a> {
        Any::from_tag_and_data(Self::TAG, &self.data)
    }

    pub fn as_u8(&self) -> Result<u8> {
        self.any().try_into()
    }

    pub fn as_u16(&self) -> Result<u16> {
        self.any().try_into()
    }

    pub fn as_u32(&self) -> Result<u32> {
        self.any().try_into()
    }

    pub fn as_u64(&self) -> Result<u64> {
        self.any().try_into()
    }

    pub fn as_u128(&self) -> Result<u128> {
        self.any().try_into()
    }

    pub fn as_i8(&self) -> Result<i8> {
        self.any().try_into()
    }

    pub fn as_i16(&self) -> Result<i16> {
        self.any().try_into()
    }

    pub fn as_i32(&self) -> Result<i32> {
        self.any().try_into()
    }

    pub fn as_i64(&self) -> Result<i64> {
        self.any().try_into()
    }

    pub fn as_i128(&self) -> Result<i128> {
        self.any().try_into()
    }

    #[cfg(feature = "bigint")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
    pub fn as_bigint(&self) -> BigInt {
        BigInt::from_bytes_be(Sign::Plus, &self.data)
    }

    #[cfg(feature = "bigint")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
    pub fn as_biguint(&self) -> BigUint {
        BigUint::from_bytes_be(&self.data)
    }
}

impl<'a> AsRef<[u8]> for Integer<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for Integer<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Integer<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        Ok(Integer {
            data: any.into_cow(),
        })
    }
}

impl<'a> CheckDerConstraints for Integer<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for Integer<'a> {
    const TAG: Tag = Tag::Integer;
}

#[cfg(test)]
mod tests {
    use crate::FromDer;

    // Vectors from Section 5.7 of:
    // https://luca.ntop.org/Teaching/Appunti/asn1.html
    pub(crate) const I0_BYTES: &[u8] = &[0x02, 0x01, 0x00];
    pub(crate) const I127_BYTES: &[u8] = &[0x02, 0x01, 0x7F];
    pub(crate) const I128_BYTES: &[u8] = &[0x02, 0x02, 0x00, 0x80];
    pub(crate) const I256_BYTES: &[u8] = &[0x02, 0x02, 0x01, 0x00];
    pub(crate) const INEG128_BYTES: &[u8] = &[0x02, 0x01, 0x80];
    pub(crate) const INEG129_BYTES: &[u8] = &[0x02, 0x02, 0xFF, 0x7F];

    // Additional vectors
    pub(crate) const I255_BYTES: &[u8] = &[0x02, 0x02, 0x00, 0xFF];
    pub(crate) const I32767_BYTES: &[u8] = &[0x02, 0x02, 0x7F, 0xFF];
    pub(crate) const I65535_BYTES: &[u8] = &[0x02, 0x03, 0x00, 0xFF, 0xFF];
    pub(crate) const INEG32768_BYTES: &[u8] = &[0x02, 0x02, 0x80, 0x00];

    #[test]
    fn decode_i8() {
        assert_eq!(0, i8::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, i8::from_der(I127_BYTES).unwrap().1);
        assert_eq!(-128, i8::from_der(INEG128_BYTES).unwrap().1);
    }

    #[test]
    fn decode_i16() {
        assert_eq!(0, i16::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, i16::from_der(I127_BYTES).unwrap().1);
        assert_eq!(128, i16::from_der(I128_BYTES).unwrap().1);
        assert_eq!(255, i16::from_der(I255_BYTES).unwrap().1);
        assert_eq!(256, i16::from_der(I256_BYTES).unwrap().1);
        assert_eq!(32767, i16::from_der(I32767_BYTES).unwrap().1);
        assert_eq!(-128, i16::from_der(INEG128_BYTES).unwrap().1);
        assert_eq!(-129, i16::from_der(INEG129_BYTES).unwrap().1);
        assert_eq!(-32768, i16::from_der(INEG32768_BYTES).unwrap().1);
    }

    #[test]
    fn decode_u8() {
        assert_eq!(0, u8::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, u8::from_der(I127_BYTES).unwrap().1);
        assert_eq!(255, u8::from_der(I255_BYTES).unwrap().1);
    }

    #[test]
    fn decode_u16() {
        assert_eq!(0, u16::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, u16::from_der(I127_BYTES).unwrap().1);
        assert_eq!(255, u16::from_der(I255_BYTES).unwrap().1);
        assert_eq!(256, u16::from_der(I256_BYTES).unwrap().1);
        assert_eq!(32767, u16::from_der(I32767_BYTES).unwrap().1);
        assert_eq!(65535, u16::from_der(I65535_BYTES).unwrap().1);
    }

    /// Integers must be encoded with a minimum number of octets
    #[test]
    fn reject_non_canonical() {
        assert!(i8::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(i16::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(u8::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(u16::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
    }
}
