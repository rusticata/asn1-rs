use asn1_rs::*;
use hex_literal::hex;

fn derive_derparser_default() {
    #[derive(Debug, PartialEq, Eq, DerParserSet)]
    // #[debug_derive]
    pub struct AADefault {
        #[default(0)]
        a: u32,
    }

    // Ok: value present
    let input = Input::from_slice(&hex!("31 04 020200aa"));
    let (rem, res) = AADefault::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AADefault { a: 0xaa });

    // Ok: value absent
    let input = Input::from_slice(&hex!("31 00"));
    let (rem, res) = AADefault::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AADefault { a: 0 });

    // Ok: value absent, remaining bytes
    let input = Input::from_slice(&hex!("31 04 040200aa"));
    let (rem, res) = AADefault::parse_der(input).expect("parsing failed");
    // NOTE: rem is empty, because there are remaining bytes in _content_ (ignored by default)
    assert!(rem.is_empty());
    assert_eq!(res, AADefault { a: 0 });
}

fn main() {
    derive_derparser_default();
}
