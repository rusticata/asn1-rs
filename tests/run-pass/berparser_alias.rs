use asn1_rs::*;
use hex_literal::hex;

#[derive(Debug, PartialEq, Eq)]
//
#[derive(BerParserAlias)]
// #[debug_derive]
pub struct AnyAlias<'a>(Any<'a>);

#[derive(Debug, PartialEq, Eq)]
//
#[derive(BerParserAlias)]
// #[debug_derive]
pub struct U32Alias(u32);

fn main() {
    // Ok: object with expected content
    let input = Input::from_slice(&hex!("020200aa"));
    let (rem, res) = <AnyAlias>::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert!(res.0.as_u32().is_ok());

    // Ok: object with expected content
    let input = Input::from_slice(&hex!("020200aa"));
    let (rem, res) = <U32Alias>::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, U32Alias(0xaa));

    // Fail: not the expected content
    let input = Input::from_slice(&hex!("040200aa"));
    let _e = <U32Alias>::parse_ber(input).expect_err("parsing should fail");
}
