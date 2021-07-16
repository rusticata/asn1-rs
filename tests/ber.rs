use asn1_rs::*;
use hex_literal::hex;
use nom::Needed;

#[test]
fn from_ber_any() {
    let input = &hex!("02 01 02 ff ff");
    let (rem, result) = Any::from_ber(input).expect("parsing failed");
    // dbg!(&result);
    assert_eq!(rem, &[0xff, 0xff]);
    assert_eq!(result.header.tag(), Tag::Integer);
}

#[test]
fn from_ber_bitstring() {
    //
    // correct DER encoding
    //
    let input = &hex!("03 04 06 6e 5d c0");
    let (rem, result) = BitString::from_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(result.unused_bits, 6);
    assert_eq!(&result.data[..], &input[3..]);
    //
    // correct encoding, but wrong padding bits (not all set to 0)
    //
    let input = &hex!("03 04 06 6e 5d e0");
    let (rem, result) = BitString::from_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(result.unused_bits, 6);
    assert_eq!(&result.data[..], &input[3..]);
    //
    // long form of length (invalid, < 127)
    //
    let input = &hex!("03 81 04 06 6e 5d c0");
    let (rem, result) = BitString::from_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(result.unused_bits, 6);
    assert_eq!(&result.data[..], &input[4..]);
}

#[test]
fn from_ber_endofcontent() {
    let input = &hex!("00 00");
    let (rem, _result) = EndOfContent::from_ber(input).expect("parsing failed");
    assert_eq!(rem, &[]);
}

#[test]
fn from_ber_generalizedtime() {
    let input = &hex!("18 0F 32 30 30 32 31 32 31 33 31 34 32 39 32 33 5A FF");
    let (rem, result) = GeneralizedTime::from_ber(input).expect("parsing failed");
    assert_eq!(rem, &[0xff]);
    #[cfg(feature = "datetime")]
    {
        use chrono::{TimeZone, Utc};
        let datetime = Utc.ymd(2002, 12, 13).and_hms(14, 29, 23);

        assert_eq!(result.utc_datetime(), datetime);
    }
    // local time with fractional seconds
    let input = b"\x18\x1019851106210627.3";
    let (rem, result) = GeneralizedTime::from_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    #[cfg(feature = "datetime")]
    {
        use chrono::{TimeZone, Utc};
        let datetime = Utc.ymd(1985, 11, 6).and_hms(21, 6, 27);
        assert_eq!(result.utc_datetime(), datetime);
        assert_eq!(result.0.millisecond, Some(3));
        assert_eq!(result.0.tz, ASN1TimeZone::Undefined);
    }
    // coordinated universal time with fractional seconds
    let input = b"\x18\x1119851106210627.3Z";
    let (rem, result) = GeneralizedTime::from_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    #[cfg(feature = "datetime")]
    {
        use chrono::{TimeZone, Utc};
        let datetime = Utc.ymd(1985, 11, 6).and_hms(21, 6, 27);
        assert_eq!(result.utc_datetime(), datetime);
        assert_eq!(result.0.millisecond, Some(3));
        assert_eq!(result.0.tz, ASN1TimeZone::Z);
    }
    // local time with fractional seconds, and with local time 5 hours retarded in relation to coordinated universal time.
    let input = b"\x18\x1519851106210627.3-0500";
    let (rem, result) = GeneralizedTime::from_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    #[cfg(feature = "datetime")]
    {
        use chrono::{TimeZone, Utc};
        let datetime = Utc.ymd(1985, 11, 6).and_hms(21, 6, 27);
        assert_eq!(result.utc_datetime(), datetime);
        assert_eq!(result.0.millisecond, Some(3));
        assert_eq!(result.0.tz, ASN1TimeZone::Offset(-1, 5, 0));
    }
}

#[test]
fn from_ber_int() {
    let input = &hex!("02 01 02 ff ff");
    let (rem, result) = u8::from_ber(input).expect("parsing failed");
    assert_eq!(result, 2);
    assert_eq!(rem, &[0xff, 0xff]);
}

#[test]
fn from_ber_octetstring() {
    let input = &hex!("04 05 41 41 41 41 41");
    let (rem, result) = OctetString::from_ber(input).expect("parsing failed");
    assert_eq!(result.as_ref(), b"AAAAA");
    assert_eq!(rem, &[]);
}

#[test]
fn from_ber_real_binary() {
    const EPSILON: f32 = 0.00001;
    // binary, base = 2
    let input = &hex!("09 03 80 ff 01 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::binary(1.0, 2, -1));
    assert!((result.f32() - 0.5).abs() < EPSILON);
    assert_eq!(rem, &[0xff, 0xff]);
    // binary, base = 2 and scale factor
    let input = &hex!("09 03 94 ff 0d ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::binary(26.0, 2, -3).with_enc_base(8));
    assert!((result.f32() - 3.25).abs() < EPSILON);
    assert_eq!(rem, &[0xff, 0xff]);
    // binary, base = 16
    let input = &hex!("09 03 a0 fe 01 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::binary(1.0, 2, -8).with_enc_base(16));
    assert!((result.f32() - 0.00390625).abs() < EPSILON);
    assert_eq!(rem, &[0xff, 0xff]);
    // binary, exponent = 0
    let input = &hex!("09 03 80 00 01 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::binary(1.0, 2, 0));
    assert!((result.f32() - 1.0).abs() < EPSILON);
    assert_eq!(rem, &[0xff, 0xff]);
    // 2 octets for exponent and negative exponent
    let input = &hex!("09 04 a1 ff 01 03 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::binary(3.0, 2, -1020).with_enc_base(16));
    let epsilon = 1e-311_f64;
    assert!((result.f64() - 2.67e-307).abs() < epsilon);
    assert_eq!(rem, &[0xff, 0xff]);
}

#[test]
fn from_ber_real_special() {
    // 0
    let input = &hex!("09 00 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::from(0.0));
    assert_eq!(rem, &[0xff, 0xff]);
    // infinity
    let input = &hex!("09 01 40 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::Infinity);
    assert_eq!(rem, &[0xff, 0xff]);
    // negative infinity
    let input = &hex!("09 01 41 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::NegInfinity);
    assert_eq!(rem, &[0xff, 0xff]);
}

#[test]
#[allow(clippy::approx_constant)]
fn from_ber_real_string() {
    // text representation, NR3
    let input = &hex!("09 07 03 33 31 34 45 2D 32 ff ff");
    let (rem, result) = Real::from_ber(input).expect("parsing failed");
    assert_eq!(result, Real::from(3.14));
    assert_eq!(rem, &[0xff, 0xff]);
}

#[test]
#[allow(clippy::approx_constant)]
fn from_ber_real_string_primitive() {
    // text representation, NR3
    let input = &hex!("09 07 03 33 31 34 45 2D 32 ff ff");
    let (rem, result) = f32::from_ber(input).expect("parsing failed");
    assert!((result - 3.14).abs() < 0.01);
    assert_eq!(rem, &[0xff, 0xff]);
}

#[test]
fn from_ber_sequence() {
    let input = &hex!("30 05 02 03 01 00 01");
    let (rem, result) = Sequence::from_ber(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
    assert_eq!(rem, &[]);
}

#[test]
fn from_ber_sequence_vec() {
    let input = &hex!("30 05 02 03 01 00 01");
    let (rem, result) = <Vec<u32>>::from_ber(input).expect("parsing failed");
    assert_eq!(&result, &[65537]);
    assert_eq!(rem, &[]);
}

#[test]
fn from_ber_iter_sequence() {
    let input = &hex!("30 0a 02 03 01 00 01 02 03 01 00 01");
    let (rem, result) = Sequence::from_ber(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
    assert_eq!(rem, &[]);
    let v = result
        .der_iter()
        .collect::<Result<Vec<u32>>>()
        .expect("could not iterate sequence");
    assert_eq!(&v, &[65537, 65537]);
}

#[test]
fn from_ber_iter_sequence_incomplete() {
    let input = &hex!("30 09 02 03 01 00 01 02 03 01 00");
    let (rem, result) = Sequence::from_ber(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
    assert_eq!(rem, &[]);
    let mut iter = result.ber_iter::<u32>();
    assert_eq!(iter.next(), Some(Ok(65537)));
    assert_eq!(iter.next(), Some(Err(Error::Incomplete(Needed::new(1)))));
    assert_eq!(iter.next(), None);
}

#[test]
fn from_ber_universalstring() {
    let input = &hex!("1C 10 00000061 00000062 00000063 00000064");
    let (rem, result) = UniversalString::from_ber(input).expect("parsing failed");
    assert_eq!(result.as_ref(), "abcd");
    assert_eq!(rem, &[]);
}
