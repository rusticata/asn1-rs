use crate::{Any, CheckDerConstraints, Error, Length, Result, Tag, Tagged};
use nom::Needed;
use std::convert::TryFrom;

mod f32;
mod f64;
pub use self::f32::*;
pub use self::f64::*;

/// ASN.1 `REAL` type
#[derive(Debug, PartialEq)]
pub enum Real {
    Binary {
        mantissa: i64,
        base: u32,
        exponent: i32,
    },
    Value(f64),
}

impl Real {
    /// Infinity (∞).
    pub const INFINITY: Real = Real::Value(f64::INFINITY);
    /// Negative infinity (−∞).
    pub const NEG_INFINITY: Real = Real::Value(f64::NEG_INFINITY);
    /// Zero
    pub const ZERO: Real = Real::Value(0.0);

    /// Create a new binary `REAL`
    #[inline]
    pub const fn binary(mantissa: i64, base: u32, exponent: i32) -> Self {
        Self::Binary {
            mantissa,
            base,
            exponent,
        }
    }

    /// Returns `true` if this value is `NaN`.
    #[inline]
    pub fn is_nan(&self) -> bool {
        match self {
            Real::Value(f) => f.is_nan(),
            _ => false,
        }
    }

    /// Returns `true` if this value is positive infinity or negative infinity, and
    /// `false` otherwise.
    #[inline]
    pub fn is_infinite(&self) -> bool {
        match self {
            Real::Value(f) => f.is_infinite(),
            _ => false,
        }
    }

    /// Returns `true` if this number is neither infinite nor `NaN`.
    #[inline]
    pub fn is_finite(&self) -> bool {
        match self {
            Real::Value(f) => f.is_finite(),
            _ => false,
        }
    }

    /// Returns the 'f64' value of this `REAL`.
    pub fn f64(&self) -> f64 {
        match self {
            Real::Binary {
                mantissa,
                base,
                exponent,
            } => {
                let f = *mantissa as f64;
                let exp = (*base as f64).powi(*exponent);
                f * exp
            }
            Real::Value(f) => *f,
        }
    }

    /// Returns the 'f32' value of this `REAL`.
    ///
    /// This functions casts the result of [`Real::f64`] to a `f32`, and loses precision.
    pub fn f32(&self) -> f32 {
        self.f64() as f32
    }
}

impl<'a> TryFrom<Any<'a>> for Real {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        let data = &any.data;
        if data.is_empty() {
            return Ok(Real::ZERO);
        }
        // code inspired from pyasn1
        let first = data[0];
        let rem = &data[1..];
        if first & 0x80 != 0 {
            // binary encoding (X.690 section 8.5.6)
            let rem = rem;
            // format of exponent
            let (n, rem) = match first & 0x03 {
                4 => {
                    let (b, rem) = rem
                        .split_first()
                        .ok_or_else(|| Error::Incomplete(Needed::new(1)))?;
                    (*b as usize, rem)
                }
                b => (b as usize + 1, rem),
            };
            if n >= rem.len() {
                return Err(any.tag().invalid_value("Invalid float value(exponent)"));
            }
            // n cannot be 0 (see the +1 above)
            let (eo, rem) = rem.split_at(n);
            // so 'eo' cannot be empty
            let mut e = if eo[0] & 0x80 != 0 { -1 } else { 0 };
            // safety check: 'eo' length must be <= container type for 'e'
            if eo.len() > 4 {
                return Err(any.tag().invalid_value("Exponent too large (REAL)"));
            }
            for b in eo {
                e = (e << 8) | (*b as i32);
            }
            // base bits
            let b = (first >> 4) & 0x03;
            let e = match b {
                // base 2
                0 => e,
                // base 8
                1 => e * 3,
                // base 16
                2 => e * 4,
                _ => return Err(any.tag().invalid_value("Illegal REAL base")),
            };
            if rem.len() > 8 {
                return Err(any.tag().invalid_value("Mantissa too large (REAL)"));
            }
            let mut p = 0;
            for b in rem {
                p = (p << 8) | (*b as i64);
            }
            // sign bit
            let p = if first & 0x40 != 0 { -p } else { p };
            // scale bits
            let sf = (first >> 2) & 0x03;
            // 2^sf: cannot overflow, sf is between 0 and 3
            let scale = 1_i64 << sf;
            let p = p * scale;
            Ok(Real::Binary {
                mantissa: p,
                base: 2,
                exponent: e,
            })
        } else if first & 0x40 != 0 {
            // special real value (X.690 section 8.5.8)
            // there shall be only one contents octet,
            if any.header.length != Length::Definite(1) {
                return Err(Error::InvalidLength);
            }
            // with values as follows
            match first {
                0x40 => Ok(Real::INFINITY),
                0x41 => Ok(Real::NEG_INFINITY),
                _ => Err(any.tag().invalid_value("Invalid float special value")),
            }
        } else {
            // decimal encoding (X.690 section 8.5.7)
            let s = std::str::from_utf8(rem)?;
            match first & 0x03 {
                0x1 => {
                    // NR1
                    match s.parse::<u32>() {
                        Err(_) => Err(any.tag().invalid_value("Invalid float string encoding")),
                        Ok(v) => Ok(Real::Value(v.into())),
                    }
                }
                0x2 /* NR2 */ | 0x3 /* NR3 */=> {
                    match s.parse::<f64>() {
                        Err(_) => Err(any.tag().invalid_value("Invalid float string encoding")),
                        Ok(v) => Ok(Real::Value(v)),
                    }
                        }
                c => {
                    return Err(any.tag().invalid_value(&format!("Invalid NR ({})", c)));
                }
            }
        }
    }
}

impl<'a> CheckDerConstraints for Real {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        // XXX more checks
        Ok(())
    }
}

impl Tagged for Real {
    const TAG: Tag = Tag::RealType;
}

impl From<f32> for Real {
    fn from(f: f32) -> Self {
        Real::Value(f.into())
    }
}

impl From<f64> for Real {
    fn from(f: f64) -> Self {
        Real::Value(f)
    }
}

impl From<Real> for f32 {
    fn from(r: Real) -> Self {
        r.f32()
    }
}

impl From<Real> for f64 {
    fn from(r: Real) -> Self {
        r.f64()
    }
}
