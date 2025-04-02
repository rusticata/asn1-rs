use asn1_rs::*;
use hex_literal::hex;

fn derive_sequence_simple() {
    #[derive(Debug, PartialEq, Eq, Sequence)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    // Ok: sequence with expected content
    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let (rem, res) = AA::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AA { a: 0xaa });

    // Fail: not constructed
    let input = Input::from_slice(&hex!("10 04 020200aa"));
    let _ = AA::parse_ber(input).expect_err("not constructed");

    // Fail: not a sequence
    let input = Input::from_slice(&hex!("31 04 020200aa"));
    let _ = AA::parse_ber(input).expect_err("not a sequence");
}

fn derive_sequence_attribute_parse() {
    //--- custom parsing function, using closure
    #[derive(Debug, PartialEq, Eq, Sequence)]
    // #[debug_derive]
    pub struct AAParseClosure {
        #[asn1(parse = "|input| Ok((input, 0xff))")]
        a: u32,
    }

    // Ok: value will be overridden by custom parser
    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let (rem, res) = AAParseClosure::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AAParseClosure { a: 0xff });

    //--- custom parsing function, using function
    fn parse_return_ff(input: Input) -> IResult<Input, u32, BerError<Input>> {
        Ok((input, 0xff))
    }

    #[derive(Debug, PartialEq, Eq, Sequence)]
    // #[debug_derive]
    pub struct AAParseFunction {
        #[asn1(parse = "parse_return_ff")]
        a: u32,
    }

    // Ok: value will be overridden by custom parser
    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let (rem, res) = AAParseFunction::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AAParseFunction { a: 0xff });
}

#[cfg(feature = "std")]
mod with_std {
    use super::*;

    fn derive_sequence_simple_encode() {
        #[derive(Debug, PartialEq, Eq, Sequence)]
        // #[debug_derive]
        pub struct AA {
            a: u32,
        }

        //--- variant 0
        let item = AA { a: 0xaa };
        let ber = item.to_ber_vec().expect("BER serialization failed");
        let der = item.to_der_vec().expect("BER serialization failed");

        // check BER encoding
        let expected = &hex!("30 04 020200aa");
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
        derive_sequence_simple_encode();
    }
}

fn main() {
    derive_sequence_simple();
    derive_sequence_attribute_parse();

    #[cfg(feature = "std")]
    {
        with_std::run_tests();
    }
}
