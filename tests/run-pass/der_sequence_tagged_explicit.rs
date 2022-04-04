fn test_tag_explicit() {
    use asn1_rs::*;
    use hex_literal::hex;

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T6 {
        #[tag_explicit(0)]
        a: u16,
    }

    let input0 = &hex!("3005 a003020103");
    let (rem, t6) = T6::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t6, T6 { a: 3 });

    let input1 = &hex!("3005 a103020103");
    T6::from_der(input1).expect_err("parsing tag 1 should fail");
}

fn test_tag_explicit_optional() {
    use asn1_rs::*;
    use hex_literal::hex;

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T0 {
        #[tag_explicit(0)]
        #[optional]
        a: Option<u16>,
        b: u16,
    }

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T1 {
        #[tag_explicit(0)]
        #[optional]
        a: Option<u16>,
    }

    // optional value present
    let input0 = &hex!("3008 a003020103 020103");
    let (rem, t0) = T0::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: Some(3), b: 3 });

    // optional value absent
    let input1 = &hex!("3003 020103");
    let (rem, t0) = T0::from_der(input1).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: None, b: 3 });

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

fn main() {
    test_tag_explicit();
    test_tag_explicit_optional();
}
