fn test_tag_implicit() {
    use asn1_rs::*;
    use hex_literal::hex;

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T0 {
        #[tag_implicit(0)]
        a: u16,
    }

    let input0 = &hex!("3003 800103");
    let (rem, t0) = T0::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: 3 });

    let input1 = &hex!("3003 810103");
    T0::from_der(input1).expect_err("parsing tag 1 should fail");
}

fn test_tag_implicit_optional() {
    use asn1_rs::*;
    use hex_literal::hex;

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T0 {
        #[tag_implicit(0)]
        #[optional]
        a: Option<u16>,
        b: u16,
    }

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T1 {
        #[tag_implicit(0)]
        #[optional]
        a: Option<u16>,
    }

    // optional value present
    let input0 = &hex!("3006 800103 020103");
    let (rem, t0) = T0::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: Some(3), b: 3 });

    // optional value absent
    let input1 = &hex!("3003 020103");
    let (rem, t0) = T0::from_der(input1).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: None, b: 3 });

    // optional value present but wrong tag
    let input_wrong_tag = &hex!("3006 810103 020103");
    T0::from_der(input_wrong_tag).expect_err("parsing should fail");

    // optional value present but invalid length
    let input_wrong_len0 = &hex!("3006 800203 020103");
    T0::from_der(input_wrong_len0).expect_err("parsing should fail");
    let input_wrong_len1 = &hex!("3006 800403 020103");
    T0::from_der(input_wrong_len1).expect_err("parsing should fail");

    // test empty input
    let input_empty = &hex!("3000");
    let (rem, t1) = T1::from_der(input_empty).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1, T1 { a: None });
}

fn test_tag_implicit_application() {
    use asn1_rs::*;
    use hex_literal::hex;

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T0 {
        #[tag_implicit(APPLICATION 0)]
        a: u16,
    }

    let input0 = &hex!("3003 400103");
    let (rem, t0) = T0::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: 3 });

    let input1 = &hex!("3003 410103");
    T0::from_der(input1).expect_err("parsing tag 1 should fail");
}

fn test_tag_implicit_private() {
    use asn1_rs::*;
    use hex_literal::hex;

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct T0 {
        #[tag_implicit(PRIVATE 0)]
        a: u16,
    }

    let input0 = &hex!("3003 c00103");
    let (rem, t0) = T0::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t0, T0 { a: 3 });

    let input1 = &hex!("3003 c10103");
    T0::from_der(input1).expect_err("parsing tag 1 should fail");
}

fn main() {
    test_tag_implicit();
    test_tag_implicit_optional();
    test_tag_implicit_application();
    test_tag_implicit_private();
}
