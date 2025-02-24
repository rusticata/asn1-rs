use asn1_rs::*;
use hex_literal::hex;

fn derive_derparser_tagged() {
    //----- using type

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSet)]
    // #[debug_derive]
    pub struct AATagEx1<'a> {
        a: TaggedExplicit<u32, BerError<Input<'a>>, 0>,
    }

    let input = Input::from_slice(&hex!("31 06 a0 04 020200aa"));
    let (rem, res) = AATagEx1::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(
        res,
        AATagEx1 {
            a: TaggedValue::explicit(0xaa)
        }
    );

    //----- using attribute

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSet)]
    // #[debug_derive]
    pub struct AATagEx2 {
        #[tag_explicit(0)]
        a: u32,
    }

    let input = Input::from_slice(&hex!("31 06 a0 04 020200aa"));
    let (rem, res) = AATagEx2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AATagEx2 { a: 0xaa });
}

fn derive_derparser_opttagged() {
    //----- using type

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSet)]
    // #[debug_derive]
    pub struct AATagEx1<'a> {
        a: Option<TaggedExplicit<u32, BerError<Input<'a>>, 0>>,
    }

    let input = Input::from_slice(&hex!("31 06 a0 04 020200aa"));
    let (rem, res) = AATagEx1::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(
        res,
        AATagEx1 {
            a: Some(TaggedValue::explicit(0xaa))
        }
    );

    //----- using attribute

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSet)]
    // #[debug_derive]
    pub struct AATagEx2 {
        #[tag_explicit(0)]
        #[optional]
        a: Option<u32>,
    }

    let input = Input::from_slice(&hex!("31 06 a0 04 020200aa"));
    let (rem, res) = AATagEx2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AATagEx2 { a: Some(0xaa) });
}

fn derive_derparser_opttagged_default() {
    //----- using attribute

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(DerParserSet)]
    // #[debug_derive]
    pub struct AATagEx2 {
        #[tag_explicit(0)]
        #[default(0)]
        a: u32,
    }

    // Ok: value present
    let input = Input::from_slice(&hex!("31 06 a0 04 020200aa"));
    let (rem, res) = AATagEx2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AATagEx2 { a: 0xaa });

    // Ok: value absent
    let input = Input::from_slice(&hex!("31 00"));
    let (rem, res) = AATagEx2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AATagEx2 { a: 0 });

    // Ok: value absent (different tag), remaining bytes
    let input = Input::from_slice(&hex!("31 06 a1 04 020200aa"));
    let (rem, res) = AATagEx2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AATagEx2 { a: 0 });

    // Fail: value present, but inner type different
    let input = Input::from_slice(&hex!("31 06 a0 04 040200aa"));
    let _e = AATagEx2::parse_der(input).expect_err("parsing failed");
}

fn main() {
    derive_derparser_tagged();
    derive_derparser_opttagged();
    derive_derparser_opttagged_default();
}
