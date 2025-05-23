use asn1_rs::*;
use hex_literal::hex;

fn derive_derparser_simple() {
    #[derive(Debug, PartialEq, Eq, DerParserSequence)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    // Ok: sequence with expected content
    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let (rem, res) = AA::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AA { a: 0xaa });

    // Fail: not constructed
    let input = Input::from_slice(&hex!("10 04 020200aa"));
    let _ = AA::parse_der(input).expect_err("not constructed");

    // Fail: not a sequence
    let input = Input::from_slice(&hex!("31 04 020200aa"));
    let _ = AA::parse_der(input).expect_err("not a sequence");
}

fn derive_derparser_container() {
    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSequence)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSequence)]
    // #[debug_derive]
    pub struct BB {
        a: AA,
    }

    let input = Input::from_slice(&hex!("30 06 30 04 020200aa"));
    let (rem, res) = BB::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, BB { a: AA { a: 0xaa } });
}

fn derive_derparser_and_fromder() {
    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSequence)]
    //
    #[derive(DerSequence)]
    // #[debug_derive]
    pub struct AA1 {
        a: u32,
    }

    let bytes = &hex!("30 04 020200aa");
    let input = Input::from_slice(bytes);
    let (rem, res1) = AA1::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res1, AA1 { a: 0xaa });
    let (rem, res2) = AA1::from_der(bytes).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res1, res2);

    // //----- check opposite order of derive

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerSequence)]
    //
    #[derive(DerParserSequence)]
    // #[debug_derive]
    pub struct AA2 {
        a: u32,
    }
    let bytes = &hex!("30 04 020200aa");
    let input = Input::from_slice(bytes);
    let (rem, res1) = AA2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res1, AA2 { a: 0xaa });
    let (rem, res2) = AA2::from_der(bytes).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res1, res2);
}

fn main() {
    derive_derparser_simple();
    derive_derparser_container();
    derive_derparser_and_fromder();
}
