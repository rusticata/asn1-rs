use asn1_rs::*;
use hex_literal::hex;
use nom::Needed;

#[test]
fn from_ber_any() {
    let input = &hex!("02 01 02 ff ff");
    let (rem, result) = Any::from_ber(input).expect("parsing failed");
    // dbg!(&result);
    assert_eq!(rem, &[0xff, 0xff]);
    assert_eq!(result.header.tag, Tag::Integer);
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
fn from_ber_sequence() {
    let input = &hex!("30 05 02 03 01 00 01");
    let (rem, result) = Sequence::from_ber(input).expect("parsing failed");
    assert_eq!(result.as_ref(), &input[2..]);
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
