#![no_main]

use asn1_rs::{Any, FromBer};
use libfuzzer_sys::fuzz_target;

// `exercise_der_conversions` is only used by the DER target.
#[allow(dead_code)]
mod common;

use common::exercise_conversions;

// Parse arbitrary bytes as a BER object, then exercise every conversion on the
// result.
//
// The harness does not recurse on its own: it descends exactly one level, via
// the crate's own `parse_content_ber`. Any stack exhaustion reported here is
// therefore attributable to recursion inside asn1-rs and not to the harness.
fuzz_target!(|data: &[u8]| {
    let (_rem, any) = match Any::from_ber(data) {
        Ok(v) => v,
        Err(_) => return,
    };

    exercise_conversions(&any);

    // Content of a constructed object is itself BER.
    if let Ok((_, inner)) = any.parse_content_ber::<Any>() {
        exercise_conversions(&inner);
    }
});
