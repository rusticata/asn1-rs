use asn1_rs::{DerSequence, FromDer};
use hex_literal::hex;

#[derive(Clone, Debug, PartialEq, DerSequence)]
pub struct T1 {
    a: u32,
    b: u16,
    c: u16,
}

// sequence composed of other objects
#[derive(Debug, PartialEq, DerSequence)]
pub struct T2 {
    a: u32,
    b: T1,
}

// sequence composed of a sequence of objects
#[derive(Debug, PartialEq, DerSequence)]
pub struct T3 {
    a: Vec<T1>,
}

fn main() {
    let input = &hex!("30090201 01020102 020103");
    let mut v = vec![0x30];

    let sz = 2 * input.len() as u8;
    v.push(sz + 2);
    v.extend_from_slice(&[0x30]);
    v.push(sz);
    v.extend_from_slice(input);
    v.extend_from_slice(input);

    let (rem, t3) = T3::from_der(&v).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t3.a.len(), 2);
    let t1 = T1 { a: 1, b: 2, c: 3 };
    assert_eq!(t3, T3{ a: vec![t1.clone(), t1]});
}
