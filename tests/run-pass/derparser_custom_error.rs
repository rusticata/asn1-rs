use asn1_rs::*;
use displaydoc::Display;
use hex_literal::hex;
use thiserror::Error;

#[derive(Debug, Display, PartialEq, Error)]
pub enum MyError {
    /// Not Yet Implemented
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

fn derive_derparser_custom_error() {
    #[derive(Debug, PartialEq, Eq, DerParserSequence)]
    #[error(MyError)]
    // #[debug_derive]
    pub struct AA {
        a: u32,
    }

    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let (rem, res) = AA::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AA { a: 0xaa });
}

fn derive_derparser_map_err() {
    #[derive(Debug, PartialEq, Eq, DerParserSequence)]
    #[error(MyError)]
    // #[debug_derive]
    pub struct AA {
        #[map_err(MyError::from)]
        a: u32,
    }

    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let (rem, res) = AA::parse_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, AA { a: 0xaa });
}

fn main() {
    derive_derparser_custom_error();
    derive_derparser_map_err();
}
