use asn1_rs::*;
use hex_literal::hex;

fn derive_sequence_tag_explicit() {
    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(Sequence)]
    // #[debug_derive]
    pub struct AATagEx2 {
        #[tag_explicit(0)]
        a: u32,
    }

    // Ok: sequence with expected content
    let input = Input::from_slice(&hex!("30 06 a0 04 020200aa"));
    let (rem, res) = AATagEx2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AATagEx2 { a: 0xaa });

    // Fail: tag not constructed
    let input = Input::from_slice(&hex!("30 06 80 04 020200aa"));
    let _ = AATagEx2::parse_ber(input).expect_err("tag not constructed");

    // Fail: wrong tag
    let input = Input::from_slice(&hex!("30 06 a1 04 020200aa"));
    let _ = AATagEx2::parse_ber(input).expect_err("wrong tag");

    // Fail: correct tag but wrong content
    let input = Input::from_slice(&hex!("30 06 a1 03 0101ff"));
    let _ = AATagEx2::parse_ber(input).expect_err("correct tag but wrong content");
}

fn derive_sequence_tag_implicit() {
    #[derive(Debug, PartialEq, Eq)]
    //
    #[derive(Sequence)]
    // #[debug_derive]
    pub struct AATagIm2 {
        #[tag_implicit(0)]
        a: u32,
    }

    // Ok: sequence with expected content
    let input = Input::from_slice(&hex!("30 04 800200aa"));
    let (rem, res) = AATagIm2::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AATagIm2 { a: 0xaa });

    // Fail: tag constructed
    let input = Input::from_slice(&hex!("30 04 a00200aa"));
    let _ = AATagIm2::parse_ber(input).expect_err("tag constructed");

    // Fail: wrong tag
    let input = Input::from_slice(&hex!("30 04 810200aa"));
    let _ = AATagIm2::parse_ber(input).expect_err("wrong tag");
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    fn derive_sequence_tag_explicit_encode() {
        #[derive(Debug, PartialEq, Eq, Sequence)]
        // #[debug_derive]
        pub struct AA {
            #[tag_explicit(0)]
            a: u32,
        }

        //--- variant 0
        let item = AA { a: 0xaa };
        let ber = item.to_ber_vec().expect("BER serialization failed");
        let der = item.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = &hex!("30 06 a0 04 020200aa");
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

    fn derive_sequence_tag_implicit_encode() {
        #[derive(Debug, PartialEq, Eq, Sequence)]
        // #[debug_derive]
        pub struct AATagIm2 {
            #[tag_implicit(0)]
            a: u32,
        }

        //--- variant 0
        let item = AATagIm2 { a: 0xaa };
        let ber = item.to_ber_vec().expect("BER serialization failed");
        let der = item.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = &hex!("30 04 800200aa");
        assert_eq!(&ber, expected);
        assert_eq!(&der, expected);
        // encoding should be the same
        assert_eq!(&ber, &der);

        // parse back
        let (_, r0_ber) = AATagIm2::parse_ber(Input::from(&ber)).expect("parsing BER failed");
        let (_, r0_der) = AATagIm2::parse_der(Input::from(&der)).expect("parsing DER failed");
        assert_eq!(item, r0_ber);
        assert_eq!(item, r0_der);
    }

    pub fn run_tests() {
        derive_sequence_tag_explicit_encode();
        derive_sequence_tag_implicit_encode();
    }
}

fn main() {
    derive_sequence_tag_explicit();
    derive_sequence_tag_implicit();

    #[cfg(feature = "std")]
    {
        with_std::run_tests();
    }
}
