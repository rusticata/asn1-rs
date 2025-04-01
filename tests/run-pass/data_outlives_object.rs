use asn1_rs::{DerParser, Input, Integer, Utf8String};
use hex_literal::hex;

/// Check that the compilers allows us to build an ephemeral object from input data and return
/// part of this object
fn test_data_outlives_integer(i: &[u8]) -> Option<&[u8]> {
    let (_, i) = Integer::parse_der(Input::from(i)).unwrap();
    i.as_raw_slice()
    // This fails because the AsRef traits has different lifetimes annotations,
    // so compiler complains that returned value is referencing local variable 'i'
    // Some(i.as_ref())
}

// test the same for some string types

/// Check that the compilers allows us to build an ephemeral object from input data and return
/// part of this object
fn test_data_outlives_utf8string(i: &[u8]) -> Option<&str> {
    let (_, i) = Utf8String::parse_der(Input::from(i)).unwrap();
    i.as_raw_str()
    // This fails because the AsRef traits has different lifetimes annotations,
    // so compiler complains that returned value is referencing local variable 'i'
    // Some(i.as_ref())
}

fn main() {
    //--- Integer
    let bytes = &hex!("02 02 0123");

    let opt = test_data_outlives_integer(bytes);
    assert_eq!(opt, Some(&hex!("0123") as &[u8]));

    //--- Utf8String
    let bytes = &hex!("0c 0a 53 6f 6d 65 2d 53 74 61 74 65");

    let opt = test_data_outlives_utf8string(bytes);
    assert_eq!(opt, Some("Some-State"));
}
