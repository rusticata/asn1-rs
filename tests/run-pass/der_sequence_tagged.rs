use asn1_rs::*;
use hex_literal::hex;

// tagged without parsing annotations

#[derive(Debug, PartialEq, DerSequence)]
pub struct T1 {
    a: u32,
    b: u16,
    c: TaggedExplicit<T2, Error, 0>,
}

#[derive(Debug, PartialEq, DerSequence)]
pub struct T2 {
    a: u16,
}

// test with EXPLICIT Vec
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T3 {
    a: u32,
    b: u16,
    c: TaggedExplicit<Vec<T2>, Error, 0>,
}

// test with IMPLICIT Vec
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T4 {
    a: u32,
    b: u16,
    c: TaggedImplicit<Vec<T2>, Error, 0>,
}

// test with [0] IMPLICIT xx OPTIONAL
#[derive(Debug, PartialEq, DerSequence)]
// #[debug_derive]
pub struct T5 {
    a: u32,
    b: u16,
    // c: OptTaggedImplicit<Vec<T2>, Error, 0>,
    c0: OptTaggedImplicit<Vec<T2>, Error, 0>,
    // c0: Option<TaggedValue<Vec<T2>, Error, Implicit, {Class::CONTEXT_SPECIFIC}, 0>>,
    c1: OptTaggedImplicit<Vec<T2>, Error, 1>,
    // c1: Option<TaggedValue<Vec<T2>, Error, Implicit, {Class::CONTEXT_SPECIFIC}, 1>>,
}

fn main() {
    let input = &hex!("300d 020101 020102 a0053003020103");
    let (rem, t1) = T1::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    let c = TaggedValue::explicit(T2 { a: 3 });
    assert_eq!(t1, T1 { a: 1, b: 2, c });

    let input = &hex!("300d 020101 020102 a1053003020103");
    let (rem, t5) = T5::from_der(input).expect("parsing failed");
    assert!(rem.is_empty());
    let c1 = Some(TaggedValue::implicit(vec![T2 { a: 3 }]));
    assert_eq!(
        t5,
        T5 {
            a: 1,
            b: 2,
            c0: None,
            c1
        }
    );
}
