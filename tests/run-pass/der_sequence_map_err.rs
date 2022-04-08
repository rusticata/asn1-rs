use asn1_rs::{oid, Any, DerSequence, Error, FromDer, Oid};
use hex_literal::hex;

#[derive(Debug, PartialEq)]
pub enum MyError {
    NotYetImplemented,
}

impl From<asn1_rs::Error> for MyError {
    fn from(_: asn1_rs::Error) -> Self {
        MyError::NotYetImplemented
    }
}

// no custom error nor map_err
// just for reference
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T0 {
    pub a: u32,
}

// map_err without custom error
// especially useful if subparser does not return an `Error`
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T1 {
    #[map_err(|_| Error::BerTypeError)]
    pub a: u32,
}

// custom error, no mapping (just using Into)
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
#[error(MyError)]
pub struct T2 {
    pub a: u32,
}

// custom error and error mapping
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
#[error(MyError)]
pub struct T3 {
    #[map_err(MyError::from)]
    pub a: u32,
}

// similar to T1: subparser returns an error of type MyError,
// which is mapped to `Error`
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T4 {
    #[map_err(|_| Error::BerTypeError)]
    pub a: T2,
}

// check that if subparser returns MyError, and this struct also
// does, then no annotation is required
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
#[error(MyError)]
pub struct T5 {
    pub a: T3,
}

#[derive(Debug)]
pub enum X509Error {
    NotYetImplemented,

    DerError(Error),
}

impl From<Error> for X509Error {
    fn from(e: Error) -> Self {
        X509Error::DerError(e)
    }
}

#[derive(Debug, DerSequence)]
// #[debug_derive]
#[error(X509Error)]
pub struct AttrValue<'a> {
    pub attr_type: Oid<'a>,
    pub attr_value: Any<'a>,
}

fn main() {
    let input = &hex!("30 14 06 03 55 04 0A 13 0D 4C 65 74 27 73 20 45 6E 63 72 79 70 74");
    let (rem, v) = AttrValue::from_der(input).expect("parsing failed");

    assert!(rem.is_empty());

    assert_eq!(oid! {2.5.4.10}, v.attr_type);

    let s = v
        .attr_value
        .as_printablestring()
        .expect("could not extract printablestring");
    assert_eq!("Let's Encrypt", s.as_ref());
}
