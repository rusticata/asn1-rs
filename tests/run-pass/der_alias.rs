use asn1_rs::*;
use hex_literal::hex;

#[derive(Debug, DerAlias)]
pub struct T1<'a>(pub Any<'a>);

fn main() {
    let input = &hex!("300b 020101 020102 3003020103");
    let (rem, t1) = T1::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(t1.0.tag(), Tag::Sequence);
}
