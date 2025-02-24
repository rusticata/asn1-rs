use asn1_rs::*;
use hex_literal::hex;

fn derive_berparser_lifetime() {
    #[derive(Debug, PartialEq, Eq, BerParserSet)]
    // #[debug_derive]
    pub struct StructWithLifetime<'a> {
        a: &'a [u8],
    }

    let input = Input::from_slice(&hex!("31 04 04020102"));
    let (rem, res) = StructWithLifetime::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, StructWithLifetime { a: &[1, 2] });
}

fn derive_berparser_lifetimes() {
    #[derive(Debug, PartialEq, Eq, BerParserSet)]
    // #[debug_derive]
    pub struct StructWithTwoLifetimes<'a, 'b> {
        a: &'a [u8],
        b: &'b str,
    }

    let input = Input::from_slice(&hex!("31 08 04020102 0c023132"));
    let (rem, res) = StructWithTwoLifetimes::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(
        res,
        StructWithTwoLifetimes {
            a: &[1, 2],
            b: "12"
        }
    );
}

fn main() {
    derive_berparser_lifetime();
    derive_berparser_lifetimes();
}
