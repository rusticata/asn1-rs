use asn1_rs::*;
use hex_literal::hex;

fn derive_choice_explicit_attributes_no_der() {
    #[derive(Debug, PartialEq)]
    //
    #[derive(Choice)]
    #[tagged_explicit]
    // generate parsers only
    #[asn1(parse = "BER")]
    pub enum MyEnum {
        Val0(u8),
        Val1(String),
        Val2(u32),
        Val3(Vec<u32>),
    }

    //--- no DER parser
    let bytes = &hex!("a0 03 020108");
    // parse back
    let _ = MyEnum::parse_ber(Input::from(Input::from(bytes))).expect("parsing BER failed");
    let _ = MyEnum::parse_der(Input::from(Input::from(bytes))).expect("parsing DER failed");
}

fn main() {
    derive_choice_explicit_attributes_no_der();
}
