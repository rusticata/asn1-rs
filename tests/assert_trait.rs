use asn1_rs::*;
use nom::Input;

fn assert_trait_ber_parser<'a, I: Input<Item = u8> + 'a, T: BerParser<'a, I>>() {}

#[test]
fn assert_traits_slice() {
    assert_trait_ber_parser::<&[u8], Header>();

    assert_trait_ber_parser::<&[u8], Any>();
    assert_trait_ber_parser::<&[u8], Boolean>();
    assert_trait_ber_parser::<&[u8], bool>();
    assert_trait_ber_parser::<&[u8], Null>();
    assert_trait_ber_parser::<&[u8], ()>();
    assert_trait_ber_parser::<&[u8], EndOfContent>();
    assert_trait_ber_parser::<&[u8], Enumerated>();

    assert_trait_ber_parser::<&[u8], u8>();

    assert_trait_ber_parser::<&[u8], &str>();
    assert_trait_ber_parser::<&[u8], String>();
}

#[test]
fn assert_traits_generic() {
    fn gen_input<I: Input<Item = u8>>(_: I) {
        assert_trait_ber_parser::<I, Header>();

        assert_trait_ber_parser::<I, Any<I>>();
        assert_trait_ber_parser::<I, Boolean>();
        assert_trait_ber_parser::<I, bool>();
        assert_trait_ber_parser::<I, Null>();
        assert_trait_ber_parser::<I, ()>();
        assert_trait_ber_parser::<I, EndOfContent>();
        assert_trait_ber_parser::<I, Enumerated>();

        // assert_trait_ber_parser::<I, u8>();

        // assert_trait_ber_parser::<I, &str>();
        // assert_trait_ber_parser::<I, String>();
    }

    gen_input(&[] as &[u8]);
}
