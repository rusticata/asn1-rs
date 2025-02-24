use asn1_rs::*;
use hex_literal::hex;

fn derive_berparser_custom_error() {
    #[derive(Debug, PartialEq)]
    pub enum MyError {
        NotYetImplemented,
    }

    impl<I> nom::error::ParseError<I> for MyError {
        fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
            MyError::NotYetImplemented
        }

        fn append(_input: I, _kind: nom::error::ErrorKind, _other: Self) -> Self {
            MyError::NotYetImplemented
        }
    }

    impl<'a> From<(asn1_rs::Input<'a>, asn1_rs::Error)> for MyError {
        fn from(_value: (asn1_rs::Input<'a>, asn1_rs::Error)) -> Self {
            MyError::NotYetImplemented
        }
    }

    impl<'a> From<(asn1_rs::Input<'a>, BerError<Input<'a>>)> for MyError {
        fn from(_value: (asn1_rs::Input<'a>, BerError<Input<'a>>)) -> Self {
            MyError::NotYetImplemented
        }
    }

    impl<'a> From<BerError<Input<'a>>> for MyError {
        fn from(_value: BerError<Input<'a>>) -> Self {
            MyError::NotYetImplemented
        }
    }

    #[derive(Debug, PartialEq, Eq, BerParserSet)]
    #[error(MyError)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    let input = Input::from_slice(&hex!("31 04 020200aa"));
    let (rem, res) = AA::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AA { a: 0xaa });
}

fn derive_berparser_map_err() {
    #[derive(Debug, PartialEq)]
    pub enum MyError {
        NotYetImplemented,
    }

    impl<I> nom::error::ParseError<I> for MyError {
        fn from_error_kind(_input: I, _kind: nom::error::ErrorKind) -> Self {
            MyError::NotYetImplemented
        }

        fn append(_input: I, _kind: nom::error::ErrorKind, _other: Self) -> Self {
            MyError::NotYetImplemented
        }
    }

    impl From<Error> for MyError {
        fn from(_value: Error) -> Self {
            MyError::NotYetImplemented
        }
    }

    impl<'a> From<BerError<Input<'a>>> for MyError {
        fn from(_value: BerError<Input<'a>>) -> Self {
            MyError::NotYetImplemented
        }
    }

    #[derive(Debug, PartialEq, Eq, BerParserSet)]
    #[error(MyError)]
    // #[debug_derive]
    pub struct AA {
        #[map_err(MyError::from)]
        a: u32,
    }

    let input = Input::from_slice(&hex!("31 04 020200aa"));
    let (rem, res) = AA::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AA { a: 0xaa });
}

fn main() {
    derive_berparser_custom_error();
    derive_berparser_map_err();
}
