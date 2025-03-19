use asn1_rs::*;
use hex_literal::hex;

fn derive_alias_simple() {
    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(Alias)]
    // #[debug_derive]
    pub struct U32Alias(u32);

    // Ok: object with expected content
    let input = Input::from_slice(&hex!("020200aa"));
    let (rem, res) = <U32Alias>::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, U32Alias(0xaa));

    // Fail: not the expected content
    let input = Input::from_slice(&hex!("040200aa"));
    let _e = <U32Alias>::parse_ber(input).expect_err("parsing should fail");
}

fn derive_alias_any() {
    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(Alias)]
    // #[debug_derive]
    pub struct AnyAlias<'a>(Any<'a>);

    // Ok: object with expected content
    let input = Input::from_slice(&hex!("020200aa"));
    let (rem, res) = <AnyAlias>::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert!(res.0.as_u32().is_ok());
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    fn derive_alias_simple_encode() {
        #[derive(Debug, PartialEq, Eq)]
        //
        #[derive(Alias)]
        // #[debug_derive]
        pub struct U32Alias(u32);

        let item = U32Alias(0xaa);
        let ber = item.to_ber_vec().expect("BER serialization failed");
        let der = item.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = &hex!("020200aa");
        assert_eq!(&ber, expected);
        assert_eq!(&der, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);

        // parse back
        let (_, r0_ber) = U32Alias::parse_ber(Input::from(&ber)).expect("parsing BER failed");
        let (_, r0_der) = U32Alias::parse_der(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(item, r0_ber);
        assert_eq!(item, r0_der);
    }

    fn derive_alias_any_encode() {
        #[derive(Debug, PartialEq, Eq)]
        //
        #[derive(Alias)]
        // #[debug_derive]
        pub struct AnyAlias<'a>(Any<'a>);

        let data = &hex!("020200aa");
        let (_, any) = <Any>::parse_ber(Input::from(data)).expect("Parsing u32 failed");

        let item = AnyAlias(any);
        let ber = item.to_ber_vec().expect("BER serialization failed");
        let der = item.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = data;
        assert_eq!(&ber, expected);
        assert_eq!(&der, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);

        // parse back
        let (_, r0_ber) = AnyAlias::parse_ber(Input::from(&ber)).expect("parsing BER failed");
        let (_, r0_der) = AnyAlias::parse_der(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(item, r0_ber);
        assert_eq!(item, r0_der);
    }

    pub fn run_tests() {
        derive_alias_simple_encode();
        derive_alias_any_encode();
    }
}

fn main() {
    derive_alias_simple();
    derive_alias_any();

    #[cfg(feature = "std")]
    {
        with_std::run_tests();
    }
}
