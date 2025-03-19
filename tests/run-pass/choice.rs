use asn1_rs::*;
use hex_literal::hex;

fn derive_choice_explicit() {
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
    let (_, r0_ber) =
        MyEnum::parse_ber(Input::from(Input::from(ber0))).expect("parsing BER failed");
    let (_, r0_der) =
        MyEnum::parse_ber(Input::from(Input::from(ber0))).expect("parsing DER failed");
    assert_eq!(r0_ber, MyEnum::Val0(8));
    assert_eq!(r0_der, MyEnum::Val0(8));

    // Fail: tag 0, content with incorrect type
    let ber0b = &hex!("a0 03 0101ff");
    let _ =
        MyEnum::parse_ber(Input::from(ber0b)).expect_err("Tag 0 invalid inner type");
    let _ =
        MyEnum::parse_ber(Input::from(ber0b)).expect_err("Tag 0 invalid inner type");

    // Fail: tag 0, valid content but outer tag not constructed
    let ber0c = &hex!("80 03 0101ff");
    let _ = MyEnum::parse_ber(Input::from(ber0c)).expect_err("Tag 0 not constructed");
    let _ = MyEnum::parse_ber(Input::from(ber0c)).expect_err("Tag 0 not constructed");
}

fn derive_choice_implicit() {
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
    let (_, r0_ber) =
        MyEnum::parse_ber(Input::from(ber0)).expect("parsing BER failed");
    let (_, r0_der) =
        MyEnum::parse_ber(Input::from(ber0)).expect("parsing DER failed");
    assert_eq!(r0_ber, expected);
    assert_eq!(r0_der, expected);

    // Fail: tag 0, valid content but outer tag constructed
    let ber0c = &hex!("a0 0108");
    let _ = MyEnum::parse_ber(Input::from(ber0c)).expect_err("Tag 0 constructed");
    let _ = MyEnum::parse_der(Input::from(ber0c)).expect_err("Tag 0 constructed");

    //--- variant 3: should be constructed (inner type is Vec<u32>)
    let ber3 = &hex!("a3 09 020110 020120 020140");
    let expected = MyEnum::Val3(vec![16, 32, 64]);
    let (_, r3_ber) =
        MyEnum::parse_ber(Input::from(ber3)).expect("parsing BER failed");
    let (_, r3_der) =
        MyEnum::parse_der(Input::from(ber3)).expect("parsing DER failed");
    assert_eq!(r3_ber, expected);
    assert_eq!(r3_der, expected);
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    fn derive_choice_explicit_encode() {
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

    fn derive_choice_implicit_encode() {
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
        let (_, r0_der) = MyEnum::parse_ber(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(v0, r0_ber);
        assert_eq!(v0, r0_der);

        //--- variant 3: should be constructed (inner type is Vec<u32>)
        let v3 = MyEnum::Val3(vec![16, 32, 64]);
        let ber = v3.to_ber_vec().expect("BER serialization failed");
        let expected = &hex!("a3 09 020110 020120 020140");
        assert_eq!(&ber, expected);
    }

    pub fn run_tests() {
        derive_choice_explicit_encode();
        derive_choice_implicit_encode();
    }
}

fn main() {
    derive_choice_explicit();
    derive_choice_implicit();

    #[cfg(feature = "std")]
    {
        with_std::run_tests();
    }
}
