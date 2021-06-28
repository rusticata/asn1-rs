use asn1_rs::*;
use hex_literal::hex;
// use nom::HexDisplay;
use std::{
    borrow::Cow,
    convert::{TryFrom, TryInto},
};

#[test]
fn to_der_length() {
    // indefinite length
    let length = Length::Indefinite;
    let v = length.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x80]);
    // definite, short form
    let length = Length::Definite(3);
    let v = length.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x03]);
    // definite, long form
    let length = Length::Definite(250);
    let v = length.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0xfa, 0x01]);
}

#[test]
fn to_der_tag() {
    // short tag, UNIVERSAL
    let v = (Class::Universal, 0, Tag(0x1a))
        .to_der_vec()
        .expect("serialization failed");
    assert_eq!(&v, &[0x1a]);
    // short tag, APPLICATION
    let v = (Class::Application, 0, Tag(0x1a))
        .to_der_vec()
        .expect("serialization failed");
    assert_eq!(&v, &[0x1a | (0b01 << 6)]);
    // short tag, constructed
    let v = (Class::Universal, 1, Tag(0x10))
        .to_der_vec()
        .expect("serialization failed");
    assert_eq!(&v, &[0x30]);
    // long tag, UNIVERSAL
    let v = (Class::Universal, 0, Tag(0x1a1a))
        .to_der_vec()
        .expect("serialization failed");
    assert_eq!(&v, &[0b1_1111, 0x9a, 0x34]);
}

#[test]
fn to_der_header() {
    // simple header
    let header = Header::new_simple(Tag::Integer);
    let v = header.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x2, 0x0]);
    // indefinite length
    let header = Header::new(Class::Universal, 0, Tag::Integer, Length::Indefinite);
    let v = header.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x2, 0x80]);
}

#[test]
fn to_der_any() {
    let header = Header::new_simple(Tag::Integer);
    let any = Any::new(header, Cow::Borrowed(&hex!("02")));
    let v = any.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x02, 0x01, 0x02]);
}

#[test]
fn to_der_any_raw() {
    let header = Header::new(Class::Universal, 0, Tag::Integer, Length::Definite(3));
    let any = Any::new(header, Cow::Borrowed(&hex!("02")));
    // to_vec should compute the length
    let v = any.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x02, 0x01, 0x02]);
    // to_vec_raw will use the header as provided
    let v = any.to_der_vec_raw().expect("serialization failed");
    assert_eq!(&v, &[0x02, 0x03, 0x02]);
}

#[test]
fn to_der_bool() {
    let v = Boolean::new(0xff)
        .to_der_vec()
        .expect("serialization failed");
    assert_eq!(&v, &[0x01, 0x01, 0xff]);
    //
    let v = false.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x01, 0x01, 0x00]);
    //
    let v = true.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x01, 0x01, 0xff]);
    // raw value (not 0 of 0xff)
    let v = Boolean::new(0x8a)
        .to_der_vec_raw()
        .expect("serialization failed");
    assert_eq!(&v, &[0x01, 0x01, 0x8a]);
}

#[test]
fn to_der_generalizedtime() {
    // date without millisecond
    let dt = ASN1DateTime::new(1999, 12, 31, 23, 59, 59, None, ASN1TimeZone::Z);
    let time = GeneralizedTime::new(dt);
    let v = time.to_der_vec().expect("serialization failed");
    assert_eq!(&v[..2], &hex!("18 0f"));
    assert_eq!(&v[2..], b"19991231235959Z");
    let (_, time2) = GeneralizedTime::from_der(&v).expect("decoding serialized object failed");
    assert!(time.eq(&time2));
    //
    // date with millisecond
    let dt = ASN1DateTime::new(1999, 12, 31, 23, 59, 59, Some(123), ASN1TimeZone::Z);
    let time = GeneralizedTime::new(dt);
    let v = time.to_der_vec().expect("serialization failed");
    assert_eq!(&v[..2], &hex!("18 13"));
    assert_eq!(&v[2..], b"19991231235959.123Z");
    let (_, time2) = GeneralizedTime::from_der(&v).expect("decoding serialized object failed");
    assert!(time.eq(&time2));
}

fn encode_decode_assert_int<T>(t: T, expected: &[u8])
where
    T: ToDer + std::fmt::Debug + Eq,
    for<'a> T: TryFrom<Integer<'a>, Error = Error>,
{
    let v = t.to_der_vec().expect("serialization failed");
    assert_eq!(&v, expected);
    let (_, obj) = Integer::from_der(&v).expect("decoding serialized object failed");
    let t2: T = obj.try_into().unwrap();
    assert_eq!(t, t2);
}

#[test]
fn to_der_integer() {
    let int = Integer::new(&hex!("02"));
    let v = int.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x02, 0x01, 0x02]);
    // from_u32
    let int = Integer::from_u32(2);
    let v = int.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &[0x02, 0x01, 0x02]);
    // impl ToDer for primitive types
    encode_decode_assert_int(2u32, &[0x02, 0x01, 0x02]);
    // signed i32 (> 0)
    encode_decode_assert_int(4, &[0x02, 0x01, 0x04]);
    // signed i32 (< 0)
    encode_decode_assert_int(-4, &[0x02, 0x05, 0x00, 0xff, 0xff, 0xff, 0xfc]);
}

#[test]
fn to_der_sequence() {
    let it = [2, 3, 4].iter();
    let seq = Sequence::from_iter_to_der(it).unwrap();
    let v = seq.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &hex!("30 09 02 01 02 02 01 03 02 01 04"));
    let (_, seq2) = Sequence::from_der(&v).expect("decoding serialized object failed");
    assert_eq!(seq, seq2);
    // Vec<T>::ToDer
    let v = vec![2, 3, 4].to_der_vec().expect("serialization failed");
    assert_eq!(&v, &hex!("30 09 02 01 02 02 01 03 02 01 04"));
}

#[test]
fn to_der_tagged_explicit() {
    let tagged = TaggedValue::new_explicit(Class::ContextSpecific, 1, 2u32);
    let v = tagged.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &hex!("a1 03 02 01 02"));
    let (_, t2) =
        TaggedValue::<Explicit, u32>::from_der(&v).expect("decoding serialized object failed");
    assert!(tagged.eq(&t2));
}

#[test]
fn to_der_tagged_implicit() {
    let tagged = TaggedValue::new_implicit(Class::ContextSpecific, 0, 1, 2u32);
    let v = tagged.to_der_vec().expect("serialization failed");
    assert_eq!(&v, &hex!("81 01 02"));
    let (_, t2) =
        TaggedValue::<Implicit, u32>::from_der(&v).expect("decoding serialized object failed");
    assert!(tagged.eq(&t2));
}

#[test]
fn to_der_utctime() {
    let dt = ASN1DateTime::new(99, 12, 31, 23, 59, 59, None, ASN1TimeZone::Z);
    let time = UtcTime::new(dt);
    let v = time.to_der_vec().expect("serialization failed");
    assert_eq!(&v[..2], &hex!("17 0d"));
    assert_eq!(&v[2..], b"991231235959Z");
    let (_, time2) = UtcTime::from_der(&v).expect("decoding serialized object failed");
    assert!(time.eq(&time2));
}
