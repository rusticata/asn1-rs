#![no_main]

use asn1_rs::{Any, FromDer};
use libfuzzer_sys::fuzz_target;

mod common;

use common::{exercise_conversions, exercise_der_conversions};

// DER counterpart of `parse_ber_any`.
//
// Separate target rather than a flag inside one target: DER adds canonicality
// checks (definite lengths, minimal encodings) that BER does not have, so the
// two reach different code and are worth keeping distinct corpora for.
fuzz_target!(|data: &[u8]| {
    let (_rem, any) = match Any::from_der(data) {
        Ok(v) => v,
        Err(_) => return,
    };

    exercise_conversions(&any);
    exercise_der_conversions(&any);

    if let Ok((_, inner)) = any.parse_content_der::<Any>() {
        exercise_conversions(&inner);
        exercise_der_conversions(&inner);
    }
});
