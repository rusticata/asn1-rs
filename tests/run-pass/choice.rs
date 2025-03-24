use asn1_rs::*;
use hex_literal::hex;

fn derive_choice_untagged() {
    #[derive(Debug, PartialEq)]
    //
    #[derive(Choice)]
    // #[debug_derive]
    pub enum UntaggedChoice {
        Val0(u8),
        Val1(String),
        Val2(Vec<u32>),
    }

    //--- variant 0
    // Ok: valid content
    let ber0 = &hex!("020108");
    let (_, r0_ber) = UntaggedChoice::parse_ber(Input::from(ber0)).expect("parsing BER failed");
    let (_, r0_der) = UntaggedChoice::parse_der(Input::from(ber0)).expect("parsing DER failed");
    assert_eq!(r0_ber, UntaggedChoice::Val0(8));
    assert_eq!(r0_der, UntaggedChoice::Val0(8));

    //--- variant 2
    let ber2a = &hex!("30 03 020101");
    let (_, r2_ber) = UntaggedChoice::parse_ber(Input::from(ber2a)).expect("parsing BER failed");
    let (_, r2_der) = UntaggedChoice::parse_der(Input::from(ber2a)).expect("parsing DER failed");
    assert_eq!(r2_ber, UntaggedChoice::Val2(vec![1]));
    assert_eq!(r2_ber, r2_der);

    // Fail: valid content but tag not constructed
    let ber2c = &hex!("10 03 020101");
    let _ = UntaggedChoice::parse_ber(Input::from(ber2c)).expect_err("Tag 0 not constructed");
    let _ = UntaggedChoice::parse_der(Input::from(ber2c)).expect_err("Tag 0 not constructed");

    //--- no variant
    // Fail: unexpected type
    let ber0b = &hex!("0101ff");
    let _ = UntaggedChoice::parse_ber(Input::from(ber0b)).expect_err("Tag 0 invalid inner type");
    let _ = UntaggedChoice::parse_der(Input::from(ber0b)).expect_err("Tag 0 invalid inner type");
}

fn derive_choice_untagged_lifetime() {
    #[derive(Debug, PartialEq)]
    //
    #[derive(Choice)]
    // #[debug_derive]
    pub enum UntaggedChoice<'a> {
        Val0(u8),
        Val1(&'a [u8]),
    }

    //--- variant 1
    // Ok: valid content
    let ber1 = &hex!("04 03 010203");
    let (_, r1_ber) = UntaggedChoice::parse_ber(Input::from(ber1)).expect("parsing BER failed");
    let (_, r1_der) = UntaggedChoice::parse_der(Input::from(ber1)).expect("parsing DER failed");
    let expected = UntaggedChoice::Val1(&[0x01, 0x02, 0x03]);
    assert_eq!(r1_ber, expected);
    assert_eq!(r1_der, expected);
}

fn derive_choice_tagged_explicit() {
    #[derive(Debug, PartialEq)]
    //
    #[derive(Choice)]
    #[tagged_explicit]
    // #[debug_derive]
    pub enum MyEnum {
        Val0(u8),
        Val1(String),
        Val2(u32),
        Val3(Vec<u32>),
    }

    //--- variant 0
    // Ok: tag 0, valid content
    let ber0 = &hex!("a0 03 020108");
    let (_, r0_ber) = MyEnum::parse_ber(Input::from(ber0)).expect("parsing BER failed");
    let (_, r0_der) = MyEnum::parse_der(Input::from(ber0)).expect("parsing DER failed");
    assert_eq!(r0_ber, MyEnum::Val0(8));
    assert_eq!(r0_der, MyEnum::Val0(8));

    // Fail: tag 0, content with incorrect type
    let ber0b = &hex!("a0 03 0101ff");
    let _ = MyEnum::parse_ber(Input::from(ber0b)).expect_err("Tag 0 invalid inner type");
    let _ = MyEnum::parse_der(Input::from(ber0b)).expect_err("Tag 0 invalid inner type");

    // Fail: tag 0, valid content but outer tag not constructed
    let ber0c = &hex!("80 03 0101ff");
    let _ = MyEnum::parse_ber(Input::from(ber0c)).expect_err("Tag 0 not constructed");
    let _ = MyEnum::parse_der(Input::from(ber0c)).expect_err("Tag 0 not constructed");
}

fn derive_choice_attribute_tag_explicit() {
    #[derive(Debug, PartialEq)]
    //
    #[derive(Choice)]
    #[tagged_explicit]
    pub enum MyEnum {
        Val0(u8),
        Val1(String),
        #[tag(4)]
        Val2(u32),
        #[tag(0x22)]
        Val3(Vec<u32>),
    }

    //--- variant 2
    // Ok: tag 4, valid content
    let ber0 = &hex!("a4 03 020108");
    let (_, r0_ber) = MyEnum::parse_ber(Input::from(ber0)).expect("parsing BER failed");
    let (_, r0_der) = MyEnum::parse_der(Input::from(ber0)).expect("parsing DER failed");
    assert_eq!(r0_ber, MyEnum::Val2(8));
    assert_eq!(r0_der, MyEnum::Val2(8));

    // Ok: tag 0x22, valid content
    let expected = MyEnum::Val3(vec![0x08]);
    let ber3 = &hex!("bf 22 05 3003020108");
    let (_, r3_ber) = MyEnum::parse_ber(Input::from(ber3)).expect("parsing BER failed");
    let (_, r3_der) = MyEnum::parse_der(Input::from(ber3)).expect("parsing DER failed");
    assert_eq!(r3_ber, expected);
    assert_eq!(r3_der, expected);

    // Fail: tag 2
    let ber0b = &hex!("a2 03 020108");
    let _ = MyEnum::parse_ber(Input::from(ber0b)).expect_err("Tag 2 is not declared");
    let _ = MyEnum::parse_der(Input::from(ber0b)).expect_err("Tag 2 is not declared");
}

fn derive_choice_tagged_implicit() {
    #[derive(Debug, PartialEq)]
    //
    #[derive(Choice)]
    #[tagged_implicit]
    // #[debug_derive]
    pub enum MyEnum {
        Val0(u8),
        Val1(String),
        Val2(u32),
        Val3(Vec<u32>),
    }

    //--- variant 0
    // Ok: tag 0, valid content
    let ber0 = &hex!("80 0108");
    let expected = MyEnum::Val0(8);
    let (_, r0_ber) = MyEnum::parse_ber(Input::from(ber0)).expect("parsing BER failed");
    let (_, r0_der) = MyEnum::parse_der(Input::from(ber0)).expect("parsing DER failed");
    assert_eq!(r0_ber, expected);
    assert_eq!(r0_der, expected);

    // Fail: tag 0, valid content but outer tag constructed
    let ber0c = &hex!("a0 0108");
    let _ = MyEnum::parse_ber(Input::from(ber0c)).expect_err("Tag 0 constructed");
    let _ = MyEnum::parse_der(Input::from(ber0c)).expect_err("Tag 0 constructed");

    //--- variant 3: should be constructed (inner type is Vec<u32>)
    let ber3 = &hex!("a3 09 020110 020120 020140");
    let expected = MyEnum::Val3(vec![16, 32, 64]);
    let (_, r3_ber) = MyEnum::parse_ber(Input::from(ber3)).expect("parsing BER failed");
    let (_, r3_der) = MyEnum::parse_der(Input::from(ber3)).expect("parsing DER failed");
    assert_eq!(r3_ber, expected);
    assert_eq!(r3_der, expected);
}

fn derive_choice_attribute_tag_implicit() {
    #[derive(Debug, PartialEq)]
    //
    #[derive(Choice)]
    #[tagged_implicit]
    pub enum MyEnum {
        Val0(u8),
        Val1(String),
        #[tag(4)]
        Val2(u32),
        #[tag(0x22)]
        Val3(Vec<u32>),
    }

    //--- variant 2
    // Ok: tag 4, valid content
    let ber0 = &hex!("840108");
    let (_, r0_ber) = MyEnum::parse_ber(Input::from(ber0)).expect("parsing BER failed");
    let (_, r0_der) = MyEnum::parse_der(Input::from(ber0)).expect("parsing DER failed");
    assert_eq!(r0_ber, MyEnum::Val2(8));
    assert_eq!(r0_der, MyEnum::Val2(8));

    // Ok: tag 0x22, valid content
    let expected = MyEnum::Val3(vec![0x08]);
    let ber3 = &hex!("bf 22 03 020108");
    let (_, r3_ber) = MyEnum::parse_ber(Input::from(ber3)).expect("parsing BER failed");
    let (_, r3_der) = MyEnum::parse_der(Input::from(ber3)).expect("parsing DER failed");
    assert_eq!(r3_ber, expected);
    assert_eq!(r3_der, expected);

    // Fail: tag 2
    let ber0b = &hex!("820108");
    let _ = MyEnum::parse_ber(Input::from(ber0b)).expect_err("Tag 2 is not declared");
    let _ = MyEnum::parse_der(Input::from(ber0b)).expect_err("Tag 2 is not declared");
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    fn derive_choice_untagged_encode() {
        #[derive(Debug, PartialEq)]
        //
        #[derive(Choice)]
        // #[debug_derive]
        pub enum UntaggedChoice {
            Val0(u8),
            Val1(String),
            Val2(Vec<u32>),
        }

        //--- variant 0
        {
            let v0 = UntaggedChoice::Val0(8);
            let ber = v0.to_ber_vec().expect("BER serialization failed");
            let der = v0.to_der_vec().expect("BER serialization failed");

            // check BER encoding
            let expected = &hex!("020108");
            assert_eq!(&ber, expected);
            // encoding should be the same
            assert_eq!(&ber, &der);
            // parse back
            let (_, r0_ber) =
                UntaggedChoice::parse_ber(Input::from(&ber)).expect("parsing BER failed");
            let (_, r0_der) =
                UntaggedChoice::parse_der(Input::from(&der)).expect("parsing DER failed");
            assert_eq!(v0, r0_ber);
            assert_eq!(v0, r0_der);
        }

        //--- variant 2
        {
            let v2 = UntaggedChoice::Val2(vec![1]);
            let ber = v2.to_ber_vec().expect("BER serialization failed");
            let der = v2.to_der_vec().expect("BER serialization failed");

            // check BER encoding
            let expected = &hex!("30 03 020101");
            assert_eq!(&ber, expected);
            // encoding should be the same
            assert_eq!(&ber, &der);
            // parse back
            let (_, r2_ber) =
                UntaggedChoice::parse_ber(Input::from(&ber)).expect("parsing BER failed");
            let (_, r2_der) =
                UntaggedChoice::parse_der(Input::from(&der)).expect("parsing DER failed");
            assert_eq!(v2, r2_ber);
            assert_eq!(v2, r2_der);
        }
    }

    fn derive_choice_tagged_explicit_encode() {
        #[derive(Debug, PartialEq)]
        //
        #[derive(Choice)]
        #[tagged_explicit]
        // #[debug_derive]
        pub enum MyEnum {
            Val0(u8),
            Val1(String),
            Val2(u32),
            Val3(Vec<u32>),
        }

        //--- variant 0
        let v0 = MyEnum::Val0(8);
        let ber = v0.to_ber_vec().expect("BER serialization failed");
        let der = v0.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = &hex!("a0 03 020108");
        assert_eq!(&ber, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);
        // parse back
        let (_, r0_ber) = MyEnum::parse_ber(Input::from(&ber)).expect("parsing BER failed");
        let (_, r0_der) = MyEnum::parse_der(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(v0, r0_ber);
        assert_eq!(v0, r0_der);
    }

    fn derive_choice_attribute_tag_explicit_encode() {
        #[derive(Debug, PartialEq)]
        //
        #[derive(Choice)]
        #[tagged_explicit]
        pub enum MyEnum {
            Val0(u8),
            Val1(String),
            #[tag(4)]
            Val2(u32),
            #[tag(0x22)]
            Val3(Vec<u32>),
        }

        // Ok: variant 2
        let v2 = MyEnum::Val2(8);
        let ber = v2.to_ber_vec().expect("BER serialization failed");
        let der = v2.to_der_vec().expect("DER serialization failed");
        // check BER encoding
        let expected = &hex!("a4 03 020108");
        assert_eq!(&ber, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);
    }

    fn derive_choice_tagged_implicit_encode() {
        #[derive(Debug, PartialEq)]
        //
        #[derive(Choice)]
        #[tagged_implicit]
        // #[debug_derive]
        pub enum MyEnum {
            Val0(u8),
            Val1(String),
            Val2(u32),
            Val3(Vec<u32>),
        }

        //--- variant 0
        let v0 = MyEnum::Val0(8);
        let ber = v0.to_ber_vec().expect("BER serialization failed");
        let der = v0.to_der_vec().expect("DER serialization failed");
        // check BER encoding
        let expected = &hex!("80 0108");
        assert_eq!(&ber, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);
        // parse back
        let (_, r0_ber) = MyEnum::parse_ber(Input::from(&ber)).expect("parsing BER failed");
        let (_, r0_der) = MyEnum::parse_der(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(v0, r0_ber);
        assert_eq!(v0, r0_der);

        //--- variant 3: should be constructed (inner type is Vec<u32>)
        let v3 = MyEnum::Val3(vec![16, 32, 64]);
        let ber = v3.to_ber_vec().expect("BER serialization failed");
        let expected = &hex!("a3 09 020110 020120 020140");
        assert_eq!(&ber, expected);
    }

    fn derive_choice_attribute_tag_implicit_encode() {
        #[derive(Debug, PartialEq)]
        //
        #[derive(Choice)]
        #[tagged_implicit]
        pub enum MyEnum {
            Val0(u8),
            Val1(String),
            #[tag(4)]
            Val2(u32),
            #[tag(0x22)]
            Val3(Vec<u32>),
        }

        // Ok: variant 2
        let v2 = MyEnum::Val2(8);
        let ber = v2.to_ber_vec().expect("BER serialization failed");
        let der = v2.to_der_vec().expect("DER serialization failed");
        // check BER encoding
        let expected = &hex!("840108");
        assert_eq!(&ber, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);
    }

    pub fn run_tests() {
        derive_choice_untagged_encode();
        derive_choice_tagged_explicit_encode();
        derive_choice_attribute_tag_explicit_encode();
        derive_choice_tagged_implicit_encode();
        derive_choice_attribute_tag_implicit_encode();
    }
}

fn main() {
    derive_choice_untagged();
    derive_choice_untagged_lifetime();
    derive_choice_tagged_explicit();
    derive_choice_attribute_tag_explicit();
    derive_choice_tagged_implicit();
    derive_choice_attribute_tag_implicit();

    #[cfg(feature = "std")]
    {
        with_std::run_tests();
    }
}
