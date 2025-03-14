use asn1_rs::*;
use hex_literal::hex;

fn derive_tober_simple() {
    #[derive(Debug, PartialEq, Eq, BerParserSet, ToBerSet)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    let item = AA { a: 0xaa };

    let v = item.to_ber_vec().expect("serialization failed");
    let expected = &hex!("31 04 020200aa");
    assert_eq!(&v, expected);
}

fn derive_tober_container() {
    #[derive(Debug, PartialEq, Eq, BerParserSet, ToBerSet)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(BerParserSet, ToBerSet)]
    // #[debug_derive]
    pub struct BB {
        aa: AA,
    }

    let aa = AA { a: 0xaa };
    let bb = BB { aa };

    let v = bb.to_ber_vec().expect("serialization failed");
    let expected = &hex!("31 06 31 04 020200aa");
    assert_eq!(&v, expected);
}

fn derive_tober_and_toder() {
    #[derive(Debug, PartialEq, Eq, BerParserSet, ToBerSet, ToDerSet)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    let item = AA { a: 0xaa };

    let v1 = item.to_ber_vec().expect("serialization failed");
    let expected = &hex!("31 04 020200aa");
    assert_eq!(&v1, expected);
    let v2 = item.to_der_vec().expect("serialization failed");
    assert_eq!(&v1, &v2);
}

fn main() {
    derive_tober_simple();
    derive_tober_container();
    derive_tober_and_toder();
}
