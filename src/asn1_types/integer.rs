use crate::ber::bytes_to_u64;
use crate::error::*;
use crate::traits::*;
use crate::{Any, Tag};
use std::borrow::Cow;
use std::convert::TryFrom;

#[cfg(feature = "bigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
pub use num_bigint::{BigInt, BigUint, Sign};

impl<'a> TryFrom<Any<'a>> for u8 {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<u8> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        let result = any.data[0];
        Ok(result)
    }
}

impl<'a> CheckDerConstraints for u8 {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl Tagged for u8 {
    const TAG: Tag = Tag::Integer;
}

macro_rules! impl_int {
    ($ty:ty) => {
        impl<'a> TryFrom<Any<'a>> for $ty {
            type Error = Error;

            fn try_from(any: Any<'a>) -> Result<Self> {
                any.tag().assert_eq(Self::TAG)?;
                any.header.assert_primitive()?;
                let res_u64 = bytes_to_u64(&any.data)?;
                if res_u64 > (<$ty>::MAX as u64) {
                    return Err(Error::IntegerTooLarge);
                }
                let result = res_u64 as $ty;
                Ok(result)
            }
        }
        impl<'a> CheckDerConstraints for $ty {
            fn check_constraints(any: &Any) -> Result<()> {
                any.header.assert_primitive()?;
                any.header.length.assert_definite()?;
                Ok(())
            }
        }

        impl Tagged for $ty {
            const TAG: Tag = Tag::Integer;
        }
    };
}

impl_int!(u16);
impl_int!(u32);
impl_int!(u64);

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

    pub fn as_u32(&self) -> Result<u32> {
        bytes_to_u64(&self.data).and_then(|n| {
            if n > u64::from(std::u32::MAX) {
                Err(Error::IntegerTooLarge)
            } else {
                Ok(n as u32)
            }
        })
    }

    pub fn as_u64(&self) -> Result<u64> {
        bytes_to_u64(&self.data)
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
