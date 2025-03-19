use asn1_rs::*;
use hex_literal::hex;

fn derive_set_simple() {
    #[derive(Debug, PartialEq, Eq, Set)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    // Ok: set with expected content
    let input = Input::from_slice(&hex!("31 04 020200aa"));
    let (rem, res) = AA::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AA { a: 0xaa });

    // Fail: not constructed
    let input = Input::from_slice(&hex!("11 04 020200aa"));
    let _ = AA::parse_ber(input).expect_err("not constructed");

    // Fail: not a set
    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let _ = AA::parse_ber(input).expect_err("not a set");
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    fn derive_set_simple_encode() {
        #[derive(Debug, PartialEq, Eq, Set)]
        // #[debug_derive]
        pub struct AA {
            a: u32,
        }

        //--- variant 0
        let item = AA { a: 0xaa };
        let ber = item.to_ber_vec().expect("BER serialization failed");
        let der = item.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = &hex!("31 04 020200aa");
        assert_eq!(&ber, expected);
        assert_eq!(&der, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);

        // parse back
        let (_, r0_ber) = AA::parse_ber(Input::from(&ber)).expect("parsing BER failed");
        let (_, r0_der) = AA::parse_der(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(item, r0_ber);
        assert_eq!(item, r0_der);
    }

    pub fn run_tests() {
        derive_set_simple_encode();
    }
}

fn main() {
    derive_set_simple();

    #[cfg(feature = "std")]
    {
        with_std::run_tests();
    }
}
