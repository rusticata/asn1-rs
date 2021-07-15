use crate::{DynTagged, Error, Result, Tag};
#[cfg(feature = "std")]
use crate::{SerializeResult, ToDer};
use core::ops;

/// BER Object Length
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Length {
    /// Definite form (X.690 8.1.3.3)
    Definite(usize),
    /// Indefinite form (X.690 8.1.3.6)
    Indefinite,
}

impl Length {
    /// Return true if length is definite and equal to 0
    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Length::Definite(0)
    }

    /// Get length of primitive object
    #[inline]
    pub fn definite(&self) -> Result<usize> {
        match self {
            Length::Definite(sz) => Ok(*sz),
            Length::Indefinite => Err(Error::IndefiniteLengthUnexpected),
        }
    }

    /// Return true if length is definite
    #[inline]
    pub const fn is_definite(&self) -> bool {
        matches!(self, Length::Definite(_))
    }

    /// Return error if length is not definite
    #[inline]
    pub const fn assert_definite(&self) -> Result<()> {
        match self {
            Length::Definite(_) => Ok(()),
            Length::Indefinite => Err(Error::IndefiniteLengthUnexpected),
        }
    }
}

impl From<usize> for Length {
    fn from(l: usize) -> Self {
        Length::Definite(l)
    }
}

impl ops::Add<Length> for Length {
    type Output = Self;

    fn add(self, rhs: Length) -> Self::Output {
        match self {
            Length::Indefinite => self,
            Length::Definite(lhs) => match rhs {
                Length::Indefinite => self,
                Length::Definite(rhs) => Length::Definite(lhs + rhs),
            },
        }
    }
}

impl ops::Add<usize> for Length {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        match self {
            Length::Definite(lhs) => Length::Definite(lhs + rhs),
            Length::Indefinite => self,
        }
    }
}

impl ops::AddAssign<usize> for Length {
    fn add_assign(&mut self, rhs: usize) {
        match self {
            Length::Definite(ref mut lhs) => *lhs += rhs,
            Length::Indefinite => (),
        }
    }
}

impl DynTagged for Length {
    fn tag(&self) -> Tag {
        Tag(0)
    }
}

#[cfg(feature = "std")]
impl ToDer for Length {
    fn to_der_len(&self) -> Result<usize> {
        match self {
            Length::Indefinite => Ok(1),
            Length::Definite(l) => match l {
                0..=0x7f => Ok(1),
                0x80..=0xff => Ok(2),
                0x100..=0x7fff => Ok(3),
                0x8000..=0xffff => Ok(4),
                _ => Err(Error::InvalidLength),
            },
        }
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        match *self {
            Length::Indefinite => {
                let sz = writer.write(&[0b1000_0000])?;
                Ok(sz)
            }
            Length::Definite(l) => {
                if l <= 127 {
                    // Short form
                    let sz = writer.write(&[l as u8])?;
                    Ok(sz)
                } else {
                    // Long form
                    let mut sz = 0;
                    let mut val = l;
                    loop {
                        if val <= 127 {
                            sz += writer.write(&[val as u8])?;
                            return Ok(sz);
                        } else {
                            let b = (val & 0b0111_1111) as u8 | 0b1000_0000;
                            sz += writer.write(&[b])?;
                            val >>= 7;
                        }
                    }
                }
            }
        }
    }

    fn write_der_content(&self, _writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        Ok(0)
    }
}
