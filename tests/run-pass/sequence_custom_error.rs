use asn1_rs::*;
use displaydoc::Display;
use hex_literal::hex;
use thiserror::Error;

#[derive(Debug, Display, PartialEq, Error)]
pub enum MyError {
    /// Not Yet Implemented
    NotYetImplemented,
}

impl From<BerError<Input<'_>>> for MyError {
    fn from(_: BerError<Input>) -> Self {
        MyError::NotYetImplemented
    }
}

impl nom::error::ParseError<Input<'_>> for MyError {
    fn from_error_kind(_: Input, _: nom::error::ErrorKind) -> Self {
        MyError::NotYetImplemented
    }

    fn append(_: Input, _: nom::error::ErrorKind, _: Self) -> Self {
        MyError::NotYetImplemented
    }
}

#[derive(Debug, PartialEq, Sequence)]
#[error(MyError)]
// #[debug_derive]
pub struct T2 {
    pub a: u32,
}

fn derive_sequence_custom_error() {
    // Ok: sequence with expected content
    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let (rem, res) = T2::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, T2 { a: 0xaa });

    // Fail: not constructed
    let input = Input::from_slice(&hex!("10 04 020200aa"));
    let err = T2::parse_ber(input).expect_err("not constructed");
    assert_eq!(err, Err::Error(MyError::NotYetImplemented));

    // Fail: not a sequence
    let input = Input::from_slice(&hex!("31 04 020200aa"));
    let err = T2::parse_ber(input).expect_err("not a sequence");
    assert_eq!(err, Err::Error(MyError::NotYetImplemented));
}

fn derive_sequence_map_err() {
    // subparser returns an error of type MyError,
    // in this example we map this to a `BerError`
    #[derive(Debug, PartialEq, Sequence)]
    pub struct T4 {
        #[map_err(|_| BerError::new(Input::from(&[]), InnerError::Unsupported))]
        pub t2: T2,
    }

    // Ok: sequence with expected content
    let input = Input::from_slice(&hex!("30 06 30 04 020200aa"));
    let (rem, res) = T4::parse_ber(input).expect("parsing failed");
    assert!(rem.is_empty());
    assert_eq!(res, T4 { t2: T2 { a: 0xaa } });

    // Fail: inner object not a T2 sequence
    let input = Input::from_slice(&hex!("30 04 020200aa"));
    let err = T4::parse_ber(input).expect_err("not a sequence");
    if let Err::Error(e) = err {
        assert_eq!(*e.inner(), InnerError::Unsupported);
    } else {
        panic!("unexpected nom error type");
    }

    // Compile-time test: check that we can access original error input
    #[derive(Debug, PartialEq, Sequence)]
    #[debug_derive]
    pub struct T5 {
        #[map_err(|e: BerError<Input>| BerError::err_input(e.input(), InnerError::Unsupported))]
        pub a: u32,
    }
}

fn main() {
    derive_sequence_custom_error();
    derive_sequence_map_err();
}
