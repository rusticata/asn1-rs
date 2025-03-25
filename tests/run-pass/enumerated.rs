use asn1_rs::*;
use hex_literal::hex;

fn derive_enumerated() {
    #[derive(Debug, Clone, Copy, PartialEq)]
    //
    // NOTE: enum must have an integer representation + Clone + Copy
    #[derive(Enumerated)]
    pub enum MyEnum {
        Zero = 0,
        Four = 4,
        Five,
    }

    //--- variant 0
    // Ok: tag 0, valid content
    let ber0 = &hex!("0a 0100");
    let expected = MyEnum::Zero;
    let (_, r0_ber) = MyEnum::parse_ber(Input::from(ber0)).expect("parsing BER failed");
    let (_, r0_der) = MyEnum::parse_der(Input::from(ber0)).expect("parsing DER failed");
    assert_eq!(r0_ber, expected);
    assert_eq!(r0_der, expected);

    // fail: not enumerated
    let f_ber0 = &hex!("0b 0100");
    let _ = MyEnum::parse_ber(Input::from(f_ber0)).expect_err("not ENUMERATED");
    let _ = MyEnum::parse_der(Input::from(f_ber0)).expect_err("not ENUMERATED");

    //--- variant 5
    // Ok: tag 5, valid content
    let ber5 = &hex!("0a 0105");
    let expected = MyEnum::Five;
    let (_, r5_ber) = MyEnum::parse_ber(Input::from(ber5)).expect("parsing BER failed");
    let (_, r5_der) = MyEnum::parse_der(Input::from(ber5)).expect("parsing DER failed");
    assert_eq!(r5_ber, expected);
    assert_eq!(r5_der, expected);

    // fail: value not in enumerated
    let f_ber2 = &hex!("0b 0102");
    let _ = MyEnum::parse_ber(Input::from(f_ber2)).expect_err("value not in enumerated");
    let _ = MyEnum::parse_der(Input::from(f_ber2)).expect_err("value not in enumerated");
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    fn derive_enumerated_encode() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        //
        // NOTE: enum must have an integer representation + Clone + Copy
        #[derive(Enumerated)]
        pub enum MyEnum {
            Zero = 0,
            Four = 4,
            Five,
        }

        //--- variant 4

        let v4 = MyEnum::Four;
        let ber = v4.to_ber_vec().expect("BER serialization failed");
        let der = v4.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = &hex!("0a0104");
        assert_eq!(&ber, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);
        // parse back
        let (_, r4_ber) = MyEnum::parse_ber(Input::from(&ber)).expect("parsing BER failed");
        let (_, r4_der) = MyEnum::parse_der(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(v4, r4_ber);
        assert_eq!(v4, r4_der);
    }

    pub fn run_tests() {
        derive_enumerated_encode();
    }
}

fn main() {
    derive_enumerated();

    #[cfg(feature = "std")]
    {
        with_std::run_tests();
    }
}
