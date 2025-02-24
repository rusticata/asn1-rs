use asn1_rs::*;
use hex_literal::hex;

fn derive_derparser_and_berparser() {
    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSequence)]
    //
    #[derive(BerParserSequence)]
    // #[debug_derive]
    pub struct AA1 {
        a: u32,
    }

    let bytes = &hex!("30 04 020200aa");
    let (rem, res1) = AA1::parse_der(Input::from_slice(bytes)).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res1, AA1 { a: 0xaa });
    let (rem, res2) = AA1::parse_ber(Input::from_slice(bytes)).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res1, res2);
}

fn main() {
    derive_derparser_and_berparser();
}
