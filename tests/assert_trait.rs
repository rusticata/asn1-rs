use asn1_rs::*;
// use nom::Input;

fn assert_trait_ber_parser<'a, T: BerParser<'a>>() {}

#[test]
fn assert_traits_slice() {
    assert_trait_ber_parser::<Header>();

    assert_trait_ber_parser::<Any>();
    assert_trait_ber_parser::<Boolean>();
    assert_trait_ber_parser::<bool>();
    assert_trait_ber_parser::<Null>();
    assert_trait_ber_parser::<()>();
    assert_trait_ber_parser::<EndOfContent>();
    assert_trait_ber_parser::<Enumerated>();

    assert_trait_ber_parser::<u8>();
    assert_trait_ber_parser::<&str>();
    assert_trait_ber_parser::<String>();

    assert_trait_ber_parser::<OctetString>();
}
