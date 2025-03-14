use asn1_rs::*;
use hex_literal::hex;

fn derive_tober_lifetime() {
    #[derive(Debug, PartialEq, Eq, BerParserSequence, ToBerSequence)]
    // #[debug_derive]
    pub struct StructWithLifetime<'a> {
        a: &'a [u8],
    }

    let item = StructWithLifetime { a: &[0x01, 0x02] };

    let v = item.to_ber_vec().expect("serialization failed");
    let expected = &hex!("30 04 04020102");
    assert_eq!(&v, expected);
}

fn main() {
    derive_tober_lifetime();
}
