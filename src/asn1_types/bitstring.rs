use crate::{Any, CheckDerConstraints, Error, Result, Tag, Tagged};
use nom::bitvec::{order::Msb0, slice::BitSlice};
use std::{borrow::Cow, convert::TryFrom};

/// BITSTRING object
#[derive(Clone, Debug, PartialEq)]
pub struct BitString<'a> {
    pub unused_bits: u8,
    pub data: Cow<'a, [u8]>,
}

impl<'a> BitString<'a> {
    // Length must be >= 1 (first byte is number of ignored bits)
    pub const fn new(unused_bits: u8, s: &'a [u8]) -> Self {
        BitString {
            unused_bits,
            data: Cow::Borrowed(s),
        }
    }

    /// Test if bit `bitnum` is set
    pub fn is_set(&self, bitnum: usize) -> bool {
        let byte_pos = bitnum / 8;
        if byte_pos >= self.data.len() {
            return false;
        }
        let b = 7 - (bitnum % 8);
        (self.data[byte_pos] & (1 << b)) != 0
    }

    /// Constructs a shared `&BitSlice` reference over the object data.
    pub fn as_bitslice(&self) -> Option<&BitSlice<Msb0, u8>> {
        BitSlice::<Msb0, _>::from_slice(&self.data)
    }
}

impl<'a> AsRef<[u8]> for BitString<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for BitString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<BitString<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        if any.data.is_empty() {
            return Err(Error::InvalidLength);
        }
        let data = any.into_cow();
        let (unused_bits, data) = match data {
            Cow::Borrowed(s) => (s[0], Cow::Borrowed(&s[1..])),
            Cow::Owned(v) => {
                let (head, rest) = v.split_at(1);
                (head[0], Cow::Owned(rest.to_vec()))
            }
        };
        Ok(BitString { unused_bits, data })
    }
}

impl<'a> CheckDerConstraints for BitString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        // Check that padding bits are all 0 (X.690 section 11.2.1)
        match any.data.len() {
            0 => Err(Error::InvalidLength),
            1 => {
                // X.690 section 11.2.2 Note 2
                if any.data[0] == 0 {
                    Ok(())
                } else {
                    Err(Error::InvalidLength)
                }
            }
            len => {
                let unused_bits = any.data[0];
                let last_byte = any.data[len - 1];
                if last_byte.trailing_zeros() < unused_bits as u32 {
                    return Err(Error::DerConstraintFailed);
                }

                Ok(())
            }
        }
    }
}

impl<'a> Tagged for BitString<'a> {
    const TAG: Tag = Tag::BitString;
}

#[cfg(test)]
mod tests {
    use super::BitString;

    #[test]
    fn test_bitstring_is_set() {
        let obj = BitString::new(0, &[0x0f, 0x00, 0x40]);
        assert!(!obj.is_set(0));
        assert!(obj.is_set(7));
        assert!(!obj.is_set(9));
        assert!(obj.is_set(17));
    }
}
