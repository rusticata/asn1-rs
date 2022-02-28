use asn1_rs::{oid, Any, Error, FromDer, Oid, ParseResult, Sequence};
use hex_literal::hex;

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

#[derive(Debug)]
pub struct AttrValue<'a> {
    pub attr_type: Oid<'a>,
    pub attr_value: Any<'a>,
}

impl<'a> FromDer<'a, X509Error> for AttrValue<'a> {
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, X509Error> {
        Sequence::from_der_and_then(bytes, |i| {
            let (i, attr_type) = Oid::from_der(i)?;
            let (i, attr_value) = Any::from_der(i)?;
            Ok((
                i,
                Self {
                    attr_type,
                    attr_value,
                },
            ))
        })
        .map_err(nom::Err::convert)
    }
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
