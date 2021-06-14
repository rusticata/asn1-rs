use asn1_rs::*;
use hex_literal::hex;
use nom::Needed;

#[test]
fn from_der_any() {
    let input = &hex!("02 01 02 ff ff");
    let (rem, result) = Any::from_der(input).expect("parsing failed");
    // dbg!(&result);
    assert_eq!(rem, &[0xff, 0xff]);
    assert_eq!(result.header.tag, Tag::Integer);
}

#[test]
fn from_der_bitstring() {
    //
    // correct DER encoding
    //
    let input = &hex!("03 04 06 6e 5d c0");
    let (rem, result) = BitString::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(result.unused_bits, 6);
    assert_eq!(&result.data[..], &input[3..]);
    //
    // correct encoding, but wrong padding bits (not all set to 0)
    //
    let input = &hex!("03 04 06 6e 5d e0");
    let res = BitString::from_der(input);
    assert_eq!(res, Err(Err::Failure(Error::DerConstraintFailed)));
    //
    // long form of length (invalid, < 127)
    //
    let input = &hex!("03 81 04 06 6e 5d c0");
    let res = BitString::from_der(input);
    assert_eq!(res, Err(Err::Failure(Error::DerConstraintFailed)));
}

#[test]
fn from_der_bool() {
    let input = &hex!("01 01 00");
    let (rem, result) = Boolean::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(result, Boolean::FALSE);
    //
    let input = &hex!("01 01 ff");
    let (rem, result) = Boolean::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(result, Boolean::TRUE);
    //
    let input = &hex!("01 01 7f");
    let res = Boolean::from_der(input);
    assert_eq!(res, Err(Err::Failure(Error::DerConstraintFailed)));
}

#[test]
fn from_der_endofcontent() {
    let input = &hex!("00 00");
    let (rem, _result) = EndOfContent::from_der(input).expect("parsing failed");
    assert_eq!(rem, &[]);
}

#[test]
fn from_der_enumerated() {
    let input = &hex!("0a 01 02");
    let (rem, result) = Enumerated::from_der(input).expect("parsing failed");
    assert_eq!(rem, &[]);
    assert_eq!(result.0, 2);
}

#[test]
fn from_der_int() {
    let input = &hex!("02 01 02 ff ff");
    let (rem, result) = u8::from_der(input).expect("parsing failed");
    assert_eq!(result, 2);
    assert_eq!(rem, &[0xff, 0xff]);
}

#[test]
fn from_der_octetstring() {
    let input = &hex!("04 05 41 41 41 41 41");
    let (rem, result) = OctetString::from_der(input).expect("parsing failed");
    assert_eq!(result.as_ref(), b"AAAAA");
    assert_eq!(rem, &[]);
}

#[test]
fn from_der_oid() {
    let input = &hex!("06 09 2a 86 48 86 f7 0d 01 01 05");
    let (rem, result) = Oid::from_der(input).expect("parsing failed");
    let expected = Oid::from(&[1, 2, 840, 113_549, 1, 1, 5]).unwrap();
    assert_eq!(result, expected);
    assert_eq!(rem, &[]);
}

#[test]
fn from_der_relative_oid() {
    let input = &hex!("0d 04 c2 7b 03 02");
    let (rem, result) = Oid::from_der_relative(input).expect("parsing failed");
    let expected = Oid::from_relative(&[8571, 3, 2]).unwrap();
    assert_eq!(result, expected);
    assert_eq!(rem, &[]);
}

#[test]
fn from_der_sequence() {
    let input = &hex!("30 05 02 03 01 00 01");
    let (rem, result) = Sequence::from_der(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
    assert_eq!(rem, &[]);
}

#[test]
fn from_der_sequence_vec() {
    let input = &hex!("30 05 02 03 01 00 01");
    let (rem, result) = <Vec<u32>>::from_der(input).expect("parsing failed");
    assert_eq!(&result, &[65537]);
    assert_eq!(rem, &[]);
}

#[test]
fn from_der_iter_sequence_parse() {
    let input = &hex!("30 0a 02 03 01 00 01 02 03 01 00 01");
    let (rem, result) = Sequence::from_der(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
    assert_eq!(rem, &[]);
    let (rem, v) = result
        .parse(|i| {
            let (i, i1) = u32::from_der(i)?;
            let (i, i2) = u32::from_der(i)?;
            Ok((i, (i1, i2)))
        })
        .expect("parse sequence data");
    assert_eq!(v, (65537, 65537));
    assert!(rem.is_empty());
}
#[test]
fn from_der_iter_sequence() {
    let input = &hex!("30 0a 02 03 01 00 01 02 03 01 00 01");
    let (rem, result) = Sequence::from_der(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
    assert_eq!(rem, &[]);
    let v = result
        .der_iter()
        .collect::<Result<Vec<u32>>>()
        .expect("could not iterate sequence");
    assert_eq!(&v, &[65537, 65537]);
}

#[test]
fn from_der_iter_sequence_incomplete() {
    let input = &hex!("30 09 02 03 01 00 01 02 03 01 00");
    let (rem, result) = Sequence::from_der(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
    assert_eq!(rem, &[]);
    let mut iter = result.der_iter::<u32>();
    assert_eq!(iter.next(), Some(Ok(65537)));
    assert_eq!(iter.next(), Some(Err(Error::Incomplete(Needed::new(1)))));
    assert_eq!(iter.next(), None);
}

#[test]
fn from_der_opt_int() {
    let input = &hex!("02 01 02 ff ff");
    let (rem, result) = <Option<u8>>::from_der(input).expect("parsing failed");
    assert_eq!(result, Some(2));
    assert_eq!(rem, &[0xff, 0xff]);
    // non-fatal error
    let (rem, result) = <Option<IA5String>>::from_der(input).expect("parsing failed");
    assert!(result.is_none());
    assert_eq!(rem, input);
    // fatal error (correct tag, but incomplete)
    let input = &hex!("02 03 02 01");
    let res = <Option<u8>>::from_der(input);
    assert_eq!(res, Err(nom::Err::Incomplete(Needed::new(1))));
}

#[test]
fn from_der_tagged_explicit() {
    let input = &hex!("a0 03 02 01 02");
    let (rem, result) = TaggedValue::<Explicit>::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    let (rem, i) = result.parse_der::<u32>().expect("inner parsing failed");
    assert!(rem.is_empty());
    assert_eq!(i, 2);
}

#[test]
fn from_der_tagged_implicit() {
    let input = &hex!("81 04 70 61 73 73");
    let (rem, result) = TaggedValue::<Implicit>::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    let (rem, s) = result
        .parse_der::<IA5String>()
        .expect("inner parsing failed");
    assert!(rem.is_empty());
    assert_eq!(s.as_ref(), "pass");
    // try specifying the expected tag (correct tag)
    let _ = TaggedValue::<Implicit>::from_expected_tag(input, 1).expect("parsing failed");
    // try specifying the expected tag (incorrect tag)
    let _ = TaggedValue::<Implicit>::from_expected_tag(input, 2)
        .expect_err("parsing should have failed");
}
