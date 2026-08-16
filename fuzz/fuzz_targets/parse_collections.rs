#![no_main]

use std::collections::{BTreeSet, HashSet};

use asn1_rs::*;
use libfuzzer_sys::fuzz_target;

// Collection parsers, and the parsers emitted by the derive macros.
//
// The other three targets reach concrete types only through `Any::as_*` (that
// is, `BerParser::from_ber_content`) and `<T as DerParser>::from_der_content`.
// Neither of those calls `CheckDerConstraints`, so the compatibility route
// `FromDer` -> `check_constraints` -> `TryFrom<Any>`, which is the one
// `SequenceOf<T>`, `Vec<T>`, `SetOf<T>` and every field of a derived structure
// take, is not reachable from any of them. It is reachable from here.
//
// The same holds for `AnyIterator::try_parse_collect::<_, T>` with a concrete
// `T`, and for the code the proc-macros emit: nothing in the other three
// targets instantiates either.

// Old-style derive: emits `FromBer`/`FromDer`, which go through `Any`.
#[derive(Debug, PartialEq, DerSequence)]
struct SeqPlain {
    a: u32,
    b: u64,
    c: bool,
}

// SEQUENCE OF, through the blanket `Vec<T>` parser, which is built on
// `AnyIterator::try_parse_collect`.
#[derive(Debug, PartialEq, DerSequence)]
struct SeqOfInts {
    items: Vec<u32>,
}

// Nesting: a derived parser whose field is itself a derived parser, plus a
// SEQUENCE OF derived elements.
#[derive(Debug, PartialEq, DerSequence)]
struct SeqNested {
    inner: SeqOfInts,
    rest: Vec<SeqPlain>,
}

// OPTIONAL is separate codegen: it has to tell "absent" from "present but
// malformed".
#[derive(Debug, PartialEq, DerSequence)]
struct SeqOptional {
    a: Option<u32>,
    b: u32,
    c: Option<bool>,
}

#[derive(Debug, PartialEq, BerSequence)]
struct BerSeqPlain {
    a: u32,
    b: bool,
}

// SET and SET OF, old-style derive.
#[derive(Debug, PartialEq, DerSet)]
struct SetPlain {
    a: u32,
    b: bool,
}

// New-style derive: emits `BerParser`/`DerParser` directly, which is different
// codegen from `DerSequence` (the latter goes through `Any` and
// `CheckDerConstraints`).
//
// One structure per macro: `BerParserSequence` and `DerParserSequence` each
// emit their own `impl Tagged`, so deriving both on a single structure is a
// conflicting implementation (E0119).
#[derive(Debug, PartialEq, BerParserSequence)]
struct BerParserSeq {
    a: u32,
    b: u64,
}

#[derive(Debug, PartialEq, DerParserSequence)]
struct DerParserSeq {
    a: u32,
    b: u64,
}

#[derive(Debug, PartialEq, BerParserSequence)]
struct BerParserSeqOf {
    items: Vec<u32>,
}

#[derive(Debug, PartialEq, DerParserSequence)]
struct DerParserSeqOf {
    items: Vec<u32>,
}

fuzz_target!(|data: &[u8]| {
    // `AnyIterator` driven directly, in both modes.
    //
    // Bare iteration reaches the span arithmetic in `Iterator::next` that
    // `AnySequence` also uses, but without the enclosing header parse that
    // constrains the input in the other targets.
    for r in AnyIterator::<BerMode>::new(Input::from_slice(data)) {
        match r {
            Ok((span, obj)) => {
                let _ = span.len();
                let _ = span.span();
                let _ = obj.tag();
                let _ = obj.header.length();
            }
            Err(_) => break,
        }
    }
    for r in AnyIterator::<DerMode>::new(Input::from_slice(data)) {
        match r {
            Ok((span, obj)) => {
                let _ = span.len();
                let _ = span.span();
                let _ = obj.tag();
                let _ = obj.header.length();
            }
            Err(_) => break,
        }
    }

    // `try_collect`: collecting into a container of `Any`.
    {
        let mut it = AnyIterator::<BerMode>::new(Input::from_slice(data));
        let _ = it.try_collect::<Vec<Any>>();
    }
    {
        let mut it = AnyIterator::<DerMode>::new(Input::from_slice(data));
        let _ = it.try_collect::<Vec<Any>>();
    }

    // `try_parse_collect`: collecting into a container of a concrete element
    // type. This is what `SequenceOf<T>`, `SetOf<T>`, `Vec<T>`, `HashSet<T>`
    // and `BTreeSet<T>` are all built on, and what the derive macros emit for a
    // SEQUENCE OF field.
    macro_rules! collect_as {
        ($($ty:ty),* $(,)?) => {
            $(
                {
                    let mut it = AnyIterator::<BerMode>::new(Input::from_slice(data));
                    let _ = it.try_parse_collect::<Vec<$ty>, $ty>();
                }
                {
                    let mut it = AnyIterator::<DerMode>::new(Input::from_slice(data));
                    let _ = it.try_parse_collect::<Vec<$ty>, $ty>();
                }
            )*
        };
    }
    collect_as!(
        u8,
        u32,
        u64,
        i32,
        i64,
        bool,
        Integer<'_>,
        Oid<'_>,
        OctetString<'_>,
        Utf8String<'_>,
        Ia5String<'_>,
        BitString,
        Real,
        Null,
        Enumerated,
        Any<'_>,
    );

    // Top-level SEQUENCE OF and SET OF parsers. `from_ber`/`from_der` are the
    // compatibility route, so these are the calls that reach
    // `check_constraints`.
    macro_rules! seq_of {
        ($($ty:ty),* $(,)?) => {
            $(
                let _ = SequenceOf::<$ty>::from_ber(data);
                let _ = SequenceOf::<$ty>::from_der(data);
                let _ = SetOf::<$ty>::from_ber(data);
                let _ = SetOf::<$ty>::from_der(data);
                let _ = <Vec<$ty>>::from_ber(data);
                let _ = <Vec<$ty>>::from_der(data);
            )*
        };
    }
    seq_of!(
        u32,
        u64,
        i64,
        bool,
        Integer<'_>,
        Oid<'_>,
        OctetString<'_>,
        Real,
        Any<'_>
    );

    // The hashed and ordered collectors need `Eq`/`Ord`, so they take their own
    // list.
    let _ = <HashSet<u32>>::from_ber(data);
    let _ = <HashSet<u32>>::from_der(data);
    let _ = <BTreeSet<u32>>::from_ber(data);
    let _ = <BTreeSet<u32>>::from_der(data);
    let _ = AnySequence::parse_ber(Input::from_slice(data));
    let _ = AnySequence::parse_der(Input::from_slice(data));
    let _ = <AnySet>::parse_ber(Input::from_slice(data));
    let _ = <AnySet>::parse_der(Input::from_slice(data));

    // Derive-generated parsers.
    let _ = SeqPlain::from_ber(data);
    let _ = SeqPlain::from_der(data);
    let _ = SeqOfInts::from_ber(data);
    let _ = SeqOfInts::from_der(data);
    let _ = SeqNested::from_ber(data);
    let _ = SeqNested::from_der(data);
    let _ = SeqOptional::from_ber(data);
    let _ = SeqOptional::from_der(data);
    let _ = BerSeqPlain::from_ber(data);
    let _ = SetPlain::from_ber(data);
    let _ = SetPlain::from_der(data);

    // New-style derive goes through `BerParser`/`DerParser` rather than `Any`.
    let _ = BerParserSeq::parse_ber(Input::from_slice(data));
    let _ = DerParserSeq::parse_der(Input::from_slice(data));
    let _ = BerParserSeqOf::parse_ber(Input::from_slice(data));
    let _ = DerParserSeqOf::parse_der(Input::from_slice(data));
});
