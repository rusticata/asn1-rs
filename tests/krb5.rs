//! Test implementation for Kerberos v5
//!
//! This is mostly used to verify that required types and functions are implemented,
//! and that provided API is convenient.

use asn1_rs::*;
use hex_literal::hex;

const PRINCIPAL_NAME: &[u8] = &hex!("30 81 11 a0 03 02 01 00 a1 0a 30 81 07 1b 05 4a 6f 6e 65 73");

/// PrincipalName   ::= SEQUENCE {
///         name-type       [0] Int32,
///         name-string     [1] SEQUENCE OF KerberosString
/// }
#[derive(Debug, PartialEq)]
pub struct PrincipalName {
    pub name_type: NameType,
    pub name_string: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameType(pub i32);

#[test]
fn krb5_principalname() {
    let input = PRINCIPAL_NAME;
    let (rem, res) = parse_principalname(input).expect("parsing failed");
    assert!(rem.is_empty());
    let expected = PrincipalName {
        name_type: NameType(0),
        name_string: vec!["Jones".to_string()],
    };
    assert_eq!(res, expected);
}

fn parse_principalname(bytes: &[u8]) -> ParseResult<PrincipalName> {
    // XXX in the example above, PRINCIPAL_NAME does not respect DER constraints (length is using long form while < 127)
    let (rem, seq) = Sequence::from_ber(bytes)?;
    seq.parse_ref(|input| {
        //
        let (i, t) = parse_der_tagged_explicit::<_, u32>(0)(input)?;
        let name_type = t.inner;
        // dbg!(name_type);
        let name_type = NameType(name_type as i32);
        let (_, t) = parse_der_tagged_explicit::<_, KerberosStringList>(1)(i)?;
        let name_string = t.inner.iter().map(|s| s.string()).collect();
        // dbg!(&name_string);
        Ok((
            rem,
            PrincipalName {
                name_type,
                name_string,
            },
        ))
    })
}

// KerberosString  ::= GeneralString (IA5String)
pub type KerberosString<'a> = GeneralString<'a>;

pub type KerberosStringList<'a> = Vec<KerberosString<'a>>;
