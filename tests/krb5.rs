#![cfg(feature = "std")]

//! Test implementation for Kerberos v5
//!
//! This is mostly used to verify that required types and functions are implemented,
//! and that provided API is convenient.

use asn1_rs::*;
use hex_literal::hex;
use nom::Parser;

/// PrincipalName   ::= SEQUENCE {
///         name-type       [0] Int32,
///         name-string     [1] SEQUENCE OF KerberosString
/// }
#[derive(Debug, PartialEq, Eq)]
pub struct PrincipalName {
    pub name_type: NameType,
    pub name_string: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameType(pub i32);

// KerberosString  ::= GeneralString (IA5String)
pub type KerberosString<'a> = GeneralString<'a>;

pub type KerberosStringList<'a> = Vec<KerberosString<'a>>;

impl Tagged for PrincipalName {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::Sequence;
}

impl<'a> FromDer<'a> for PrincipalName {
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        // XXX in the example above, PRINCIPAL_NAME does not respect DER constraints (length is using long form while < 127)
        let (rem, seq) = Sequence::from_ber(bytes)?;
        seq.and_then(|data| {
            let input = &data;
            let (i, t) = parse_der_tagged_explicit::<_, u32, _>(0).parse(input)?;
            let name_type = t.inner;
            let name_type = NameType(name_type as i32);
            let (_, t) = parse_der_tagged_explicit::<_, KerberosStringList, _>(1).parse(i)?;
            let name_string = t.inner.iter().map(|s| s.string()).collect();
            Ok((
                rem,
                PrincipalName {
                    name_type,
                    name_string,
                },
            ))
        })
    }
}

impl ToDer for PrincipalName {
    type Encoder = Constructed;

    // allow the `sz` alone on last line
    #[allow(clippy::let_and_return)]
    fn der_content_len(&self) -> Length {
        let sz1 =  self.name_type.0.der_total_len() + 2 /* tagged explicit */;
        let sz2 = self.name_string.der_total_len() + 2 /* tagged explicit */;
        sz1 + sz2
    }

    fn der_write_content<W: std::io::Write>(&self, target: &mut W) -> SerializeResult<usize> {
        // build DER sequence content
        let sz1 = self
            .name_type
            .0
            .explicit(Class::ContextSpecific, 0)
            .write_der(target)?;

        let sz2 = self
            .name_string
            .iter()
            .map(|s| KerberosString::from(s.as_ref()))
            .collect::<Vec<_>>()
            .explicit(Class::ContextSpecific, 1)
            .write_der(target)?;

        Ok(sz1 + sz2)
    }

    fn der_tag_info(&self) -> (Class, bool, Tag) {
        (self.class(), self.constructed(), self.tag())
    }
}

#[test]
fn krb5_principalname() {
    let input = &hex!("30 81 11 a0 03 02 01 00 a1 0a 30 81 07 1b 05 4a 6f 6e 65 73");
    let (rem, res) = PrincipalName::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    let expected = PrincipalName {
        name_type: NameType(0),
        name_string: vec!["Jones".to_string()],
    };
    assert_eq!(res, expected);
}

#[test]
fn to_der_krb5_principalname() {
    let principal = PrincipalName {
        name_type: NameType(0),
        name_string: vec!["Jones".to_string()],
    };
    let v = PrincipalName::to_der_vec(&principal).expect("serialization failed");
    // std::fs::write("/tmp/out.bin", &v).unwrap();
    let (_, principal2) = PrincipalName::from_der(&v).expect("parsing failed");
    assert!(principal.eq(&principal2));
}
