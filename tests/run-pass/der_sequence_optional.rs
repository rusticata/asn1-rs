use asn1_rs::*;
use hex_literal::hex;

// XXX limitation:
//   when OPTIONAL only (and not tagged), parser is eager:
//   if there are several values of the same type, the first one
//   goes to the OPTIONAL value (and will fail if the next value is not present and mandatory)

#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T0 {
    a: u16,
    #[optional]
    b: Option<u16>,
}

#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T1 {
    #[tag_explicit(0)]
    #[optional]
    a: Option<u16>,
}

fn main() {
    // optional value present
    let input0 = &hex!("3006 020103 020103");
    let (rem, t0) = T0::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: 3, b: Some(3) });

    // optional value absent
    let input1 = &hex!("3003 020103");
    let (rem, t0) = T0::from_der(input1).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: 3, b: None });

    // optional value present but wrong tag
    let input_wrong_tag = &hex!("3008 a103020103 020103");
    T0::from_der(input_wrong_tag).expect_err("parsing should fail");

    // optional value present but invalid length
    let input_wrong_len0 = &hex!("3008 a002020103 020103");
    T0::from_der(input_wrong_len0).expect_err("parsing should fail");
    let input_wrong_len1 = &hex!("3008 a003020403 020103");
    T0::from_der(input_wrong_len1).expect_err("parsing should fail");

    // test empty input
    let input_empty = &hex!("3000");
    let (rem, t1) = T1::from_der(input_empty).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1, T1 { a: None });
}
