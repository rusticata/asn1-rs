// Conversion surface shared by the BER and DER targets.
//
// Declared as `mod common;` by each target that needs it. Only the files listed
// as `[[bin]]` in Cargo.toml are compiled as targets, so this file is built
// solely as a submodule of those.

use asn1_rs::*;

// Exercise every `Any -> concrete type` conversion.
//
// Each of these returns a `Result`, so any panic reached from here is a crate
// bug rather than API misuse.
//
// Note: `Any::as_*` is built on `BerParser::from_ber_content`, so this covers
// BER content parsing only, for both targets. DER content parsing is a
// different code path and is covered by `exercise_der_conversions`.
pub fn exercise_conversions(any: &Any<'_>) {
    let _ = any.as_anysequence();
    let _ = any.as_bitstring();
    let _ = any.as_bmpstring();
    let _ = any.as_bool();
    let _ = any.as_boolean();
    let _ = any.as_embedded_pdv();
    let _ = any.as_endofcontent();
    let _ = any.as_enumerated();
    let _ = any.as_generalizedtime();
    let _ = any.as_generalstring();
    let _ = any.as_graphicstring();
    let _ = any.as_ia5string();
    let _ = any.as_integer();
    let _ = any.as_null();
    let _ = any.as_numericstring();
    let _ = any.as_objectdescriptor();
    let _ = any.as_octetstring();
    let _ = any.as_oid();
    let _ = any.as_printablestring();
    let _ = any.as_real();
    let _ = any.as_relative_oid();
    let _ = any.as_sequence();
    let _ = any.as_set();
    let _ = any.as_str();
    let _ = any.as_string();
    let _ = any.as_teletexstring();
    let _ = any.as_universalstring();
    let _ = any.as_utctime();
    let _ = any.as_utf8string();
    let _ = any.as_videotexstring();
    let _ = any.as_visiblestring();

    // Tag-independent string accessors take a different path than the
    // tag-checked ones above.
    let _ = any.as_any_str();
    let _ = any.as_any_string();

    // INTEGER -> primitive: one code path per width, and the documented
    // `IntegerTooLarge` failure makes the boundary worth hitting.
    let _ = any.as_i8();
    let _ = any.as_i16();
    let _ = any.as_i32();
    let _ = any.as_i64();
    let _ = any.as_i128();
    let _ = any.as_u8();
    let _ = any.as_u16();
    let _ = any.as_u32();
    let _ = any.as_u64();
    let _ = any.as_u128();
}

// Decode the content of `any` as each concrete type, using the DER parser.
//
// `DerParser::from_der_content` enforces the canonicality restrictions of
// X.690 sections 9-11 that `BerParser::from_ber_content` does not, so these are
// distinct code paths from the `as_*` methods above and are not reachable
// through them.
//
// The tag is deliberately not checked first: `from_der_content` is documented
// as not checking the header tag (it cannot, because of IMPLICIT tagging), so
// feeding it mismatched content is in-contract and must not panic.
pub fn exercise_der_conversions(any: &Any<'_>) {
    macro_rules! try_der {
        ($($ty:ty),* $(,)?) => {
            $( let _ = <$ty as DerParser>::from_der_content(&any.header, any.data.clone()); )*
        };
    }

    try_der!(
        AnySequence<'_>,
        BitString,
        BmpString<'_>,
        bool,
        Boolean,
        EmbeddedPdv<'_>,
        // no `EndOfContent`: it exists only to terminate an indefinite length,
        // which DER forbids, so it has no `DerParser` impl.
        Enumerated,
        GeneralizedTime,
        GeneralString<'_>,
        GraphicString<'_>,
        Ia5String<'_>,
        Integer<'_>,
        Null,
        NumericString<'_>,
        ObjectDescriptor<'_>,
        OctetString<'_>,
        Oid<'_>,
        PrintableString<'_>,
        Real,
        Sequence<'_>,
        Set<'_>,
        TeletexString<'_>,
        UniversalString<'_>,
        UtcTime,
        Utf8String<'_>,
        VideotexString<'_>,
        VisibleString<'_>,
        i8,
        i16,
        i32,
        i64,
        i128,
        u8,
        u16,
        u32,
        u64,
        u128,
    );
}
