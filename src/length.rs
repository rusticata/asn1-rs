use core::iter::Sum;
use core::ops;

use crate::{DynTagged, Error, InnerError, Result, Tag};

/// BER Object Length
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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

    /// Get length of primitive object
    #[inline]
    pub fn definite_inner(&self) -> Result<usize, InnerError> {
        match self {
            Length::Definite(sz) => Ok(*sz),
            Length::Indefinite => Err(InnerError::IndefiniteLengthUnexpected),
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

    /// Return error if length is not definite
    #[inline]
    pub const fn assert_definite_inner(&self) -> Result<(), InnerError> {
        match self {
            Length::Definite(_) => Ok(()),
            Length::Indefinite => Err(InnerError::IndefiniteLengthUnexpected),
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
                Length::Indefinite => rhs,
                Length::Definite(rhs) => Length::Definite(lhs + rhs),
            },
        }
    }
}

impl ops::AddAssign<Length> for Length {
    fn add_assign(&mut self, rhs: Length) {
        match (*self, rhs) {
            (Length::Definite(lhs), Length::Definite(r)) => *self = Length::Definite(lhs + r),
            (Length::Indefinite, _) => (),
            (_, Length::Indefinite) => *self = rhs,
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

impl ops::Add<Length> for usize {
    type Output = Length;

    fn add(self, rhs: Length) -> Self::Output {
        match rhs {
            Length::Definite(l) => Length::Definite(self + l),
            Length::Indefinite => rhs,
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

impl Sum<Length> for Length {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Length::Definite(0), |a, b| a + b)
    }
}

impl<'a> Sum<&'a Length> for Length {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Length::Definite(0), |a, b| a + *b)
    }
}

impl DynTagged for Length {
    fn tag(&self) -> Tag {
        Tag(0)
    }

    fn accept_tag(_: Tag) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    /// Generic and coverage tests
    #[test]
    fn methods_length() {
        let l = Length::from(2);
        assert_eq!(l.definite(), Ok(2));
        assert!(l.assert_definite().is_ok());

        let l = Length::Indefinite;
        assert!(l.definite().is_err());
        assert!(l.assert_definite().is_err());

        let l = Length::from(2);
        assert_eq!(l + 2, Length::from(4));
        assert_eq!(l + Length::Indefinite, Length::Indefinite);

        let l = Length::Indefinite;
        assert_eq!(l + 2, Length::Indefinite);

        let l = Length::from(2);
        assert_eq!(l + Length::from(2), Length::from(4));

        let l = Length::Indefinite;
        assert_eq!(l + Length::from(2), Length::Indefinite);

        let mut l = Length::from(2);
        l += 2;
        assert_eq!(l.definite(), Ok(4));

        let mut l = Length::Indefinite;
        l += 2;
        assert_eq!(l, Length::Indefinite);
    }
}
