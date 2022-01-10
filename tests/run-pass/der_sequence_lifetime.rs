use asn1_rs::{DerSequence, FromDer};
use hex_literal::hex;

// simple lifetime
#[derive(Debug, PartialEq, DerSequence)]
pub struct T1<'a> {
    a: u32,
    b: u16,
    c: u16,
    d: &'a str,
}

// lifetime with a constraint
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T2<'a> where 'a: 'static {
    a: u32,
    b: u16,
    c: u16,
    d: &'a str,
}

fn main() {
    let input = &hex!("300f0201 01020102 020103 0c0461626364");
    let (rem, t1) = T1::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1, T1 { a: 1, b: 2, c: 3, d: "abcd" });
}