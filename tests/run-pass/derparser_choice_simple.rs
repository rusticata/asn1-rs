use asn1_rs::*;
use hex_literal::hex;

fn derive_derparser_choice_explicit() {
    #[derive(Debug)]
    //
    #[derive(DerParserChoice)]
    // #[debug_derive]
    pub enum MyEnum {
        Val0(u8),
        Val1(String),
        Val2(u32),
    }

    // Ok: variant with tag 0
    let hex_v0a = &hex!("a0 03 020108");
    let (rem, res) = MyEnum::parse_der(Input::from(hex_v0a)).expect("parsing failed");
    assert!(rem.is_empty());
    assert!(matches!(res, MyEnum::Val0(8)));

    // Fail: variant with tag 0 but wrong type
    let hex_v0b = &hex!("a0 03 0101ff");
    let _ = MyEnum::parse_der(Input::from(hex_v0b)).expect_err("CHOICE variant wrong type");

    // Ok: variant with tag 1
    let hex_v1 = &hex!("a1 03 0c0141");
    let (rem, res) = MyEnum::parse_der(Input::from(hex_v1)).expect("parsing failed");
    assert!(rem.is_empty());
    assert!(matches!(res, MyEnum::Val1(s) if &s == "A"));

    // Fail: wrong tag
    let hex_v5 = &hex!("a5 03 0101ff");
    let _ = MyEnum::parse_der(Input::from(hex_v5)).expect_err("CHOICE wrong tag");

    // Fail: correct tag and inner value, but not constructed
    let hex_v0c = &hex!("80 03 020108");
    let _ = MyEnum::parse_der(Input::from(hex_v0c)).expect_err("CHOICE EXPLICIT not constructed");
}

fn derive_derparser_choice_implicit() {
    #[derive(Debug)]
    //
    #[derive(DerParserChoice)]
    #[tagged_implicit]
    // #[debug_derive]
    pub enum MyEnum {
        Val0(u8),
        Val1(String),
        Val2(u32),
    }

    // Ok: variant with tag 0
    let hex_v0a = &hex!("80 0108");
    let (rem, res) = MyEnum::parse_der(Input::from(hex_v0a)).expect("parsing failed");
    assert!(rem.is_empty());
    assert!(matches!(res, MyEnum::Val0(8)));

    // Ok: variant with tag 1
    let hex_v1 = &hex!("81 0141");
    let (rem, res) = MyEnum::parse_der(Input::from(hex_v1)).expect("parsing failed");
    assert!(rem.is_empty());
    assert!(matches!(res, MyEnum::Val1(s) if &s == "A"));

    // Fail: wrong tag
    let hex_v5 = &hex!("85 01ff");
    let _ = MyEnum::parse_der(Input::from(hex_v5)).expect_err("CHOICE wrong tag");

    // Fail: correct tag and inner value, but constructed
    let hex_v0c = &hex!("a0 0108");
    let _ = MyEnum::parse_der(Input::from(hex_v0c)).expect_err("CHOICE IMPLICIT bas constructed");
}

fn main() {
    derive_derparser_choice_explicit();
    derive_derparser_choice_implicit();
}
