#![no_main]

use asn1_rs::{Any, FromBer, FromDer};
use libfuzzer_sys::fuzz_target;

// Differential invariant: DER is a restriction of BER (X.690), so any input
// that parses as DER must also parse as BER and yield the same header,
// content and remainder.
//
// This is an oracle target rather than a panic hunt: a failure here may be a
// deliberate divergence rather than a bug. It is kept separate from the two
// parse targets so those can be taken without having to accept this invariant.
fuzz_target!(|data: &[u8]| {
    let (der_rem, der_any) = match Any::from_der(data) {
        Ok(v) => v,
        Err(_) => return,
    };

    let (ber_rem, ber_any) = match Any::from_ber(data) {
        Ok(v) => v,
        Err(e) => panic!("input parses as DER but not as BER: {e:?}"),
    };

    assert_eq!(
        der_any.header, ber_any.header,
        "header mismatch (DER vs BER)"
    );
    assert_eq!(der_any.data, ber_any.data, "content mismatch (DER vs BER)");
    assert_eq!(der_rem, ber_rem, "remainder mismatch (DER vs BER)");
});
