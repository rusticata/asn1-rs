use crate::*;
use alloc::format;
use nom::Input as _;

mod f32;
mod f64;

/// ASN.1 `REAL` type
///
/// # Limitations
///
/// When encoding binary values, only base 2 is supported
#[derive(Debug, PartialEq)]
pub enum Real {
    /// Non-special values
    Binary {
        mantissa: f64,
        base: u32,
        exponent: i32,
        enc_base: u8,
    },
    /// Infinity (∞).
    Infinity,
    /// Negative infinity (−∞).
    NegInfinity,
    /// Zero
    Zero,
}

impl Real {
    /// Create a new `REAL` from the `f64` value.
    pub fn new(f: f64) -> Self {
        if f.is_infinite() {
            if f.is_sign_positive() {
                Self::Infinity
            } else {
                Self::NegInfinity
            }
        } else if f.abs() == 0.0 {
            Self::Zero
        } else {
            let mut e = 0;
            let mut f = f;
            while f.fract() != 0.0 {
                f *= 10.0_f64;
                e -= 1;
            }
            Real::Binary {
                mantissa: f,
                base: 10,
                exponent: e,
                enc_base: 10,
            }
            .normalize_base10()
        }
    }

    pub const fn with_enc_base(self, enc_base: u8) -> Self {
        match self {
            Real::Binary {
                mantissa,
                base,
                exponent,
                ..
            } => Real::Binary {
                mantissa,
                base,
                exponent,
                enc_base,
            },
            e => e,
        }
    }

    fn normalize_base10(self) -> Self {
        match self {
            Real::Binary {
                mantissa,
                base: 10,
                exponent,
                enc_base: _enc_base,
            } => {
                let mut m = mantissa;
                let mut e = exponent;
                while m.abs() > f64::EPSILON && m.rem_euclid(10.0).abs() < f64::EPSILON {
                    m /= 10.0;
                    e += 1;
                }
                Real::Binary {
                    mantissa: m,
                    base: 10,
                    exponent: e,
                    enc_base: _enc_base,
                }
            }
            _ => self,
        }
    }

    /// Create a new binary `REAL`
    #[inline]
    pub const fn binary(mantissa: f64, base: u32, exponent: i32) -> Self {
        Self::Binary {
            mantissa,
            base,
            exponent,
            enc_base: 2,
        }
    }

    /// Returns `true` if this value is positive infinity or negative infinity, and
    /// `false` otherwise.
    #[inline]
    pub fn is_infinite(&self) -> bool {
        matches!(self, Real::Infinity | Real::NegInfinity)
    }

    /// Returns `true` if this number is not infinite.
    #[inline]
    pub fn is_finite(&self) -> bool {
        matches!(self, Real::Zero | Real::Binary { .. })
    }

    /// Returns the 'f64' value of this `REAL`.
    ///
    /// Returned value is a float, and may be infinite.
    pub fn f64(&self) -> f64 {
        match self {
            Real::Binary {
                mantissa,
                base,
                exponent,
                ..
            } => {
                let f = mantissa;
                let exp = (*base as f64).powi(*exponent);
                f * exp
            }
            Real::Zero => 0.0_f64,
            Real::Infinity => f64::INFINITY,
            Real::NegInfinity => f64::NEG_INFINITY,
        }
    }

    /// Returns the 'f32' value of this `REAL`.
    ///
    /// This functions casts the result of [`Real::f64`] to a `f32`, and loses precision.
    pub fn f32(&self) -> f32 {
        self.f64() as f32
    }
}

impl_tryfrom_any!(Real);

impl<'i> BerParser<'i> for Real {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 8.5.1)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        let r = decode_real(header, input.as_bytes2())
            .map_err(|e| BerError::nom_err_input(&input, e))?;

        // decode_real consumes all bytes
        Ok((input.take_from(input.len()), r))
    }
}

impl<'i> DerParser<'i> for Real {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        Self::from_ber_content(header, input)
    }
}

impl CheckDerConstraints for Real {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        // XXX more checks
        Ok(())
    }
}

impl DerAutoDerive for Real {}

impl Tagged for Real {
    const TAG: Tag = Tag::RealType;
}

fn decode_real(header: &Header, bytes: &[u8]) -> Result<Real, InnerError> {
    if bytes.is_empty() {
        return Ok(Real::Zero);
    }
    // code inspired from pyasn1
    let first = bytes[0];
    let rem = &bytes[1..];

    if first & 0x80 != 0 {
        // binary encoding (X.690 section 8.5.6)
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
            return Err(InnerError::invalid_value(
                header.tag,
                "Invalid float value(exponent)",
            ));
        }
        // n cannot be 0 (see the +1 above)
        let (eo, rem) = rem.split_at(n);
        // so 'eo' cannot be empty
        let mut e = if eo[0] & 0x80 != 0 { -1 } else { 0 };
        // safety check: 'eo' length must be <= container type for 'e'
        if eo.len() > 4 {
            return Err(InnerError::invalid_value(
                header.tag,
                "Exponent too large (REAL)",
            ));
        }
        for b in eo {
            e = (e << 8) | (*b as i32);
        }
        // base bits
        let b = (first >> 4) & 0x03;
        let _enc_base = match b {
            0 => 2,
            1 => 8,
            2 => 16,
            _ => {
                return Err(InnerError::invalid_value(
                    header.tag,
                    "Illegal REAL encoding base",
                ))
            }
        };
        let e = match b {
            // base 2
            0 => e,
            // base 8
            1 => e * 3,
            // base 16
            2 => e * 4,
            _ => return Err(InnerError::invalid_value(header.tag, "Illegal REAL base")),
        };
        if rem.len() > 8 {
            return Err(InnerError::invalid_value(
                header.tag,
                "Mantissa too large (REAL)",
            ));
        }
        let mut p = 0;
        for b in rem {
            p = (p << 8) | (*b as i64);
        }
        // sign bit
        let p = if first & 0x40 != 0 { -p } else { p };
        // scale bits
        let sf = (first >> 2) & 0x03;
        let p = match sf {
            0 => p as f64,
            sf => {
                // 2^sf: cannot overflow, sf is between 0 and 3
                let scale = 2_f64.powi(sf as _);
                (p as f64) * scale
            }
        };
        Ok(Real::Binary {
            mantissa: p,
            base: 2,
            exponent: e,
            enc_base: _enc_base,
        })
    } else if first & 0x40 != 0 {
        // special real value (X.690 section 8.5.8)
        // there shall be only one contents octet,
        if header.length != Length::Definite(1) {
            return Err(InnerError::InvalidLength);
        }
        // with values as follows
        match first {
            0x40 => Ok(Real::Infinity),
            0x41 => Ok(Real::NegInfinity),
            _ => Err(InnerError::invalid_value(
                header.tag,
                "Invalid float special value",
            )),
        }
    } else {
        // decimal encoding (X.690 section 8.5.7)
        let s = alloc::str::from_utf8(rem)?;
        match first & 0x03 {
            0x1 => {
                // NR1
                match s.parse::<u32>() {
                    Err(_) => Err(InnerError::invalid_value(header.tag,"Invalid float string encoding")),
                    Ok(v) => Ok(Real::new(v.into())),
                }
            }
            0x2 /* NR2 */ | 0x3 /* NR3 */=> {
                match s.parse::<f64>() {
                    Err(_) => Err(InnerError::invalid_value(header.tag,"Invalid float string encoding")),
                    Ok(v) => Ok(Real::new(v)),
                }
                    }
            c => {
                Err(InnerError::invalid_value(header.tag,&format!("Invalid NR ({})", c)))
            }
        }
    }
}

#[cfg(feature = "std")]
impl ToDer for Real {
    fn to_der_len(&self) -> Result<usize> {
        match self {
            Real::Zero => Ok(0),
            Real::Infinity | Real::NegInfinity => Ok(1),
            Real::Binary { .. } => {
                let mut sink = std::io::sink();
                let n = self
                    .write_der_content(&mut sink)
                    .map_err(|_| Self::TAG.invalid_value("Serialization of REAL failed"))?;
                Ok(n)
            }
        }
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let header = Header::new(
            Class::Universal,
            false,
            Self::TAG,
            Length::Definite(self.to_der_len()?),
        );
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        match self {
            Real::Zero => Ok(0),
            Real::Infinity => writer.write(&[0x40]).map_err(Into::into),
            Real::NegInfinity => writer.write(&[0x41]).map_err(Into::into),
            Real::Binary {
                mantissa,
                base,
                exponent,
                enc_base: _enc_base,
            } => {
                if *base == 10 {
                    // using character form
                    let sign = if *exponent == 0 { "+" } else { "" };
                    let s = format!("\x03{}E{}{}", mantissa, sign, exponent);
                    return writer.write(s.as_bytes()).map_err(Into::into);
                }
                if *base != 2 {
                    return Err(Self::TAG.invalid_value("Invalid base for REAL").into());
                }
                let mut first: u8 = 0x80;
                // choose encoding base
                let enc_base = *_enc_base;
                let (ms, mut m, enc_base, mut e) =
                    drop_floating_point(*mantissa, enc_base, *exponent);
                assert!(m != 0);
                if ms < 0 {
                    first |= 0x40
                };
                // exponent & mantissa normalization
                match enc_base {
                    2 => {
                        while m & 0x1 == 0 {
                            m >>= 1;
                            e += 1;
                        }
                    }
                    8 => {
                        while m & 0x7 == 0 {
                            m >>= 3;
                            e += 1;
                        }
                        first |= 0x10;
                    }
                    _ /* 16 */ => {
                        while m & 0xf == 0 {
                            m >>= 4;
                            e += 1;
                        }
                        first |= 0x20;
                    }
                }
                // scale factor
                // XXX in DER, sf is always 0 (11.3.1)
                let mut sf = 0;
                while m & 0x1 == 0 && sf < 4 {
                    m >>= 1;
                    sf += 1;
                }
                first |= sf << 2;
                // exponent length and bytes
                let len_e = match e.abs() {
                    0..=0xff => 1,
                    0x100..=0xffff => 2,
                    0x1_0000..=0xff_ffff => 3,
                    // e is an `i32` so it can't be longer than 4 bytes
                    // use 4, so `first` is ORed with 3
                    _ => 4,
                };
                first |= (len_e - 1) & 0x3;
                // write first byte
                let mut n = writer.write(&[first])?;
                // write exponent
                // special case: number of bytes from exponent is > 3 and cannot fit in 2 bits
                #[allow(clippy::identity_op)]
                if len_e == 4 {
                    let b = len_e & 0xff;
                    n += writer.write(&[b])?;
                }
                // we only need to write e.len() bytes
                let bytes = e.to_be_bytes();
                n += writer.write(&bytes[(4 - len_e) as usize..])?;
                // write mantissa
                let bytes = m.to_be_bytes();
                let mut idx = 0;
                for &b in bytes.iter() {
                    if b != 0 {
                        break;
                    }
                    idx += 1;
                }
                n += writer.write(&bytes[idx..])?;
                Ok(n)
            }
        }
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io;
    use std::io::Write;

    impl ToBer for Real {
        type Encoder = Primitive<{ Tag::RealType.0 }>;

        fn content_len(&self) -> Length {
            match self {
                Real::Zero => Length::Definite(0),
                Real::Infinity | Real::NegInfinity => Length::Definite(1),
                Real::Binary { .. } => {
                    let mut sink = io::sink();
                    let n = self.write_der_content(&mut sink).unwrap_or(0);
                    Length::Definite(n)
                }
            }
        }

        fn write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            match self {
                Real::Zero => Ok(0),
                Real::Infinity => target.write(&[0x40]).map_err(Into::into),
                Real::NegInfinity => target.write(&[0x41]).map_err(Into::into),
                Real::Binary {
                    mantissa,
                    base,
                    exponent,
                    enc_base: _enc_base,
                } => {
                    if *base == 10 {
                        // using character form
                        let sign = if *exponent == 0 { "+" } else { "" };
                        let s = format!("\x03{}E{}{}", mantissa, sign, exponent);
                        return target.write(s.as_bytes()).map_err(Into::into);
                    }
                    if *base != 2 {
                        return Err(Self::TAG.invalid_value("Invalid base for REAL").into());
                    }
                    let mut first: u8 = 0x80;
                    // choose encoding base
                    let enc_base = *_enc_base;
                    let (ms, mut m, enc_base, mut e) =
                        drop_floating_point(*mantissa, enc_base, *exponent);
                    assert!(m != 0);
                    if ms < 0 {
                        first |= 0x40
                    };
                    // exponent & mantissa normalization
                    match enc_base {
                        2 => {
                            while m & 0x1 == 0 {
                                m >>= 1;
                                e += 1;
                            }
                        }
                        8 => {
                            while m & 0x7 == 0 {
                                m >>= 3;
                                e += 1;
                            }
                            first |= 0x10;
                        }
                        _ /* 16 */ => {
                            while m & 0xf == 0 {
                                m >>= 4;
                                e += 1;
                            }
                            first |= 0x20;
                        }
                    }
                    // scale factor
                    // XXX in DER, sf is always 0 (11.3.1)
                    let mut sf = 0;
                    while m & 0x1 == 0 && sf < 4 {
                        m >>= 1;
                        sf += 1;
                    }
                    first |= sf << 2;
                    // exponent length and bytes
                    let len_e = match e.abs() {
                        0..=0xff => 1,
                        0x100..=0xffff => 2,
                        0x1_0000..=0xff_ffff => 3,
                        // e is an `i32` so it can't be longer than 4 bytes
                        // use 4, so `first` is ORed with 3
                        _ => 4,
                    };
                    first |= (len_e - 1) & 0x3;
                    // write first byte
                    let mut n = target.write(&[first])?;
                    // write exponent
                    // special case: number of bytes from exponent is > 3 and cannot fit in 2 bits
                    #[allow(clippy::identity_op)]
                    if len_e == 4 {
                        let b = len_e & 0xff;
                        n += target.write(&[b])?;
                    }
                    // we only need to write e.len() bytes
                    let bytes = e.to_be_bytes();
                    n += target.write(&bytes[(4 - len_e) as usize..])?;
                    // write mantissa
                    let bytes = m.to_be_bytes();
                    let mut idx = 0;
                    for &b in bytes.iter() {
                        if b != 0 {
                            break;
                        }
                        idx += 1;
                    }
                    n += target.write(&bytes[idx..])?;
                    Ok(n)
                }
            }
        }

        fn tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }
};

impl From<f32> for Real {
    fn from(f: f32) -> Self {
        Real::new(f.into())
    }
}

impl From<f64> for Real {
    fn from(f: f64) -> Self {
        Real::new(f)
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

#[cfg(feature = "std")]
fn drop_floating_point(m: f64, b: u8, e: i32) -> (i8, u64, u8, i32) {
    let ms = if m.is_sign_positive() { 1 } else { -1 };
    let es = if e.is_positive() { 1 } else { -1 };
    let mut m = m.abs();
    let mut e = e;
    //
    if b == 8 {
        m *= 2_f64.powi((e.abs() / 3) * es);
        e = (e.abs() / 3) * es;
    } else if b == 16 {
        m *= 2_f64.powi((e.abs() / 4) * es);
        e = (e.abs() / 4) * es;
    }
    //
    while m.abs() > f64::EPSILON {
        if m.fract() != 0.0 {
            m *= b as f64;
            e -= 1;
        } else {
            break;
        }
    }
    (ms, m as u64, b, e)
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, Input, Real};

    #[test]
    fn parse_ber_real_binary() {
        const EPSILON: f32 = 0.00001;
        // binary, base = 2
        let input = Input::from(&hex!("09 03 80 ff 01 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::binary(1.0, 2, -1));
        assert!((result.f32() - 0.5).abs() < EPSILON);
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        // binary, base = 2 and scale factor
        let input = Input::from(&hex!("09 03 94 ff 0d ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::binary(26.0, 2, -3).with_enc_base(8));
        assert!((result.f32() - 3.25).abs() < EPSILON);
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        // binary, base = 16
        let input = Input::from(&hex!("09 03 a0 fe 01 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::binary(1.0, 2, -8).with_enc_base(16));
        assert!((result.f32() - 0.00390625).abs() < EPSILON);
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        // binary, exponent = 0
        let input = Input::from(&hex!("09 03 80 00 01 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::binary(1.0, 2, 0));
        assert!((result.f32() - 1.0).abs() < EPSILON);
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        // 2 octets for exponent and negative exponent
        let input = Input::from(&hex!("09 04 a1 ff 01 03 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::binary(3.0, 2, -1020).with_enc_base(16));
        let epsilon = 1e-311_f64;
        assert!((result.f64() - 2.67e-307).abs() < epsilon);
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
    }

    #[test]
    fn parse_ber_real_special() {
        // 0
        let input = Input::from(&hex!("09 00 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::from(0.0));
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        // infinity
        let input = Input::from(&hex!("09 01 40 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::Infinity);
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        // negative infinity
        let input = Input::from(&hex!("09 01 41 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::NegInfinity);
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn parse_ber_real_string() {
        // text representation, NR3
        let input = Input::from(&hex!("09 07 03 33 31 34 45 2D 32 ff ff"));
        let (rem, result) = Real::parse_ber(input).expect("parsing failed");
        assert_eq!(result, Real::from(3.14));
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{Real, ToBer};

        #[test]
        fn tober_real_binary() {
            // base = 2, value = 4
            let r = Real::binary(2.0, 2, 1);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 03 80 02 01"));

            // base = 2, value = 0.5
            let r = Real::binary(0.5, 2, 0);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 03 80 ff 01"));

            // base = 2, value = 3.25, but change encoding base (8)
            let r = Real::binary(3.25, 2, 0).with_enc_base(8);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            // note: this encoding has a scale factor (not DER compliant)
            assert_eq!(&v, &hex!("09 03 94 ff 0d"));

            // base = 2, value = 0.00390625, but change encoding base (16)
            let r = Real::binary(0.00390625, 2, 0).with_enc_base(16);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            // note: this encoding has a scale factor (not DER compliant)
            assert_eq!(&v, &hex!("09 03 a0 fe 01"));

            // 2 octets for exponent, negative exponent and abs(exponent) is all 1's and fills the whole octet(s)
            let r = Real::binary(3.0, 2, -1020);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 04 81 fc 04 03"));

            // 3 octets for exponent, and
            // check that first 9 bits for exponent are not all 1's
            let r = Real::binary(1.0, 2, 262140);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 05 82 03 ff fc 01"));

            // >3 octets for exponent, and
            // mantissa < 0
            let r = Real::binary(-1.0, 2, 76354972);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 07 c3 04 04 8d 15 9c 01"));
        }

        #[test]
        fn tober_real_special() {
            // ZERO
            let r = Real::Zero;
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 00"));

            // INFINITY
            let r = Real::Infinity;
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 01 40"));

            // MINUS INFINITY
            let r = Real::NegInfinity;
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex!("09 01 41"));
        }

        #[test]
        fn tober_real_string() {
            //  non-zero value, base 10
            let r = Real::new(1.2345);
            let mut v = Vec::new();
            r.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &[&hex!("09 09 03") as &[u8], b"12345E-4"].concat());
        }
    }
}
