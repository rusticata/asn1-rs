use asn1_rs::*;
use hex_literal::hex;

#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T0 {
    #[optional]
    #[default(0)]
    a: u16,
}

#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T1 {
    #[tag_explicit(0)]
    #[optional]
    #[default(0)]
    a: u16,
}

fn main() {
    // optional value present
    let input1 = &hex!("3003 020103");
    let (rem, t0) = T0::from_der(input1).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: 3 });

    // optional value absent
    let input_empty = &hex!("3000");
    let (rem, t1) = T0::from_der(input_empty).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1, T0 { a: 0 });

    // optional value present
    let input1 = &hex!("3005 a003 020103");
    let (rem, t0) = T1::from_der(input1).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T1 { a: 3 });

    // optional value absent
    let input_empty = &hex!("3000");
    let (rem, t1) = T1::from_der(input_empty).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1, T1 { a: 0 });
}
