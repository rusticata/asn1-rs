fn test_opt_tag_explicit() {
    use asn1_rs::*;
    use hex_literal::hex;

    #[derive(Debug, PartialEq, DerSequence)]
    // #[debug_derive]
    pub struct MyStruct {
        a: u32,
        #[tag_explicit(2)]
        #[optional]
        b: BerOption<u32>,
    }

    // optional value absent
    let input0 = &hex!("3003 020103");
    let (rem, ms) = MyStruct::from_der(input0).expect("parsing failed");
    assert!(rem.is_empty());
    let expected = MyStruct {
        a: 3,
        b: BerOption::new(None),
    };
    assert_eq!(ms, expected);

    // optional value present
    let input1 = &hex!("3008 020103 a203020103");
    let (rem, ms) = MyStruct::from_der(input1).expect("parsing failed");
    assert!(rem.is_empty());
    let expected = MyStruct {
        a: 3,
        b: BerOption::new(Some(3)),
    };
    assert_eq!(ms, expected);

    // Incorrect tag: optional value just ignored
    let input2 = &hex!("3008 020103 a303020103");
    let (rem, ms) = MyStruct::from_der(input2).expect("parsing failed");
    assert!(rem.is_empty());
    let expected = MyStruct {
        a: 3,
        b: BerOption::new(None),
    };
    assert_eq!(ms, expected);

    // tag value is correct (2) but content is OctetString (4) and not Integer(2)
    let input3 = &hex!("3008 020103 a203040103");
    MyStruct::from_der(input3).expect_err("parsing tag content should fail");

    // tag correct, but parse error (non-canonical bool)
    let input3 = &hex!("3008 020103 a203010177");
    MyStruct::from_der(input3).expect_err("parsing tag content should fail");
}

fn main() {
    test_opt_tag_explicit();
}