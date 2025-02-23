use asn1_rs::*;
use hex_literal::hex;

#[derive(Debug, PartialEq, DerSequence)]
pub struct T1 {
    a: u32,
    b: u16,
    c: u16,
}

fn main() {
    let input = &hex!("30090201 01020102 020103");
    let (rem, t1) = T1::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1, T1 { a: 1, b: 2, c: 3 });

    let (rem, t1) = T1::parse_der(input.into()).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1, T1 { a: 1, b: 2, c: 3 });
}