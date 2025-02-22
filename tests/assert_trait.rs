use asn1_rs::*;
// use nom::Input;

fn assert_trait_ber_parser<'a, T: BerParser<'a>>() {}
fn assert_trait_der_parser<'a, T: DerParser<'a>>() {}

/// Compile-time verification that all supported types implement FromBer
#[test]
fn assert_traits_berparser() {
    macro_rules! test_assert {
        ($($_type:ty),*) => {
            $( test_assert!(_SINGLE $_type); )*
        };

        (_SINGLE $_type:ty) => {
            assert_trait_ber_parser::<$_type>();
        };
    }

    test_assert!(Header, Any);

    test_assert!(BitString);
    test_assert!(Boolean, bool);
    test_assert!(Null, ());

    assert_trait_ber_parser::<EndOfContent>();
    assert_trait_ber_parser::<Enumerated>();

    assert_trait_ber_parser::<Integer>();
    test_assert!(u8, u16, u32, u64, u128);
    test_assert!(i8, i16, i32, i64, i128);
    test_assert!(isize, usize);

    test_assert!(GeneralizedTime, UtcTime);

    assert_trait_ber_parser::<Oid>();

    test_assert!(Real, f32, f64);

    test_assert!(Sequence, Set);

    test_assert!(&str, String);
    test_assert!(
        BmpString,
        GeneralString,
        GraphicString,
        Ia5String,
        NumericString,
        ObjectDescriptor,
        PrintableString,
        TeletexString,
        UniversalString,
        Utf8String,
        VideotexString,
        VisibleString
    );

    //------ compound types

    // NOTE: trait bounds require the *old* trait FromBer
    // after migration, this should be empty
    #[allow(dead_code)]
    fn compound_wrapper_requiring_fromber<'a, T: FromBer<'a>>(_: T) {
        test_assert!(SetOf<T>);

        // NOTE: trait bounds require the *old* trait FromBer, with specific additional trait bounds
        #[cfg(feature = "std")]
        #[allow(dead_code)]
        fn compound_wrapper_requiring_fromber_ord<'a, T: FromBer<'a> + Ord>(_: T) {
            use std::collections::BTreeSet;
            test_assert!(BTreeSet<T>);
        }
        #[cfg(feature = "std")]
        #[allow(dead_code)]
        fn compound_wrapper_requiring_fromber_hash_eq<'a, T: FromBer<'a> + std::hash::Hash + Eq>(
            _: T,
        ) {
            use std::collections::HashSet;
            test_assert!(HashSet<T>);
        }
    }

    // test traits that should require BerParser
    #[allow(dead_code)]
    fn compound_wrapper<'a, T: BerParser<'a>>(_: T) {
        test_assert!(Option<T>);

        test_assert!(Vec<T>, SequenceOf<T>);

        // TODO: test for custom error types
        type E<'a> = BerError<Input<'a>>;
        test_assert!(TaggedExplicit<T, E, 0>);
        test_assert!(TaggedValue<T, E, Explicit, {Class::Application as u8}, 0>);

        #[allow(dead_code)]
        fn compound_wrapper_requiring_berparser_tagged<'a, T: BerParser<'a> + Tagged>(_: T) {
            // TODO: test for custom error types
            type E<'a> = BerError<Input<'a>>;
            test_assert!(TaggedImplicit<T, E, 0>);
            test_assert!(TaggedValue<T, E, Implicit, {Class::Application as u8}, 0>);
        }
    }
}

/// Compile-time verification that all supported types implement FromDer
#[test]
fn assert_traits_derparser() {
    macro_rules! test_assert {
        ($($_type:ty),*) => {
            $( test_assert!(_SINGLE $_type); )*
        };

        (_SINGLE $_type:ty) => {
            assert_trait_der_parser::<$_type>();
        };
    }

    test_assert!(Header, Any);

    // test_assert!(BitString);
    test_assert!(Boolean, bool);
    // test_assert!(Null, ());

    assert_trait_ber_parser::<EndOfContent>();
    assert_trait_ber_parser::<Enumerated>();

    assert_trait_ber_parser::<Integer>();
    test_assert!(u8, u16, u32, u64, u128);
    test_assert!(i8, i16, i32, i64, i128);
    test_assert!(isize, usize);

    // test_assert!(GeneralizedTime, UtcTime);

    assert_trait_ber_parser::<Oid>();

    // test_assert!(Real, f32, f64);

    // test_assert!(Sequence, Set);

    // test_assert!(&str, String);
    // test_assert!(
    //     BmpString,
    //     GeneralString,
    //     GraphicString,
    //     Ia5String,
    //     NumericString,
    //     ObjectDescriptor,
    //     PrintableString,
    //     TeletexString,
    //     UniversalString,
    //     Utf8String,
    //     VideotexString,
    //     VisibleString
    // );

    // //------ compound types

    // // NOTE: trait bounds require the *old* trait FromBer
    // // after migration, this should be empty
    // #[allow(dead_code)]
    // fn compound_wrapper_requiring_fromber<'a, T: FromBer<'a>>(_: T) {
    //     test_assert!(SetOf<T>);

    //     // NOTE: trait bounds require the *old* trait FromBer, with specific additional trait bounds
    //     #[cfg(feature = "std")]
    //     #[allow(dead_code)]
    //     fn compound_wrapper_requiring_fromber_ord<'a, T: FromBer<'a> + Ord>(_: T) {
    //         use std::collections::BTreeSet;
    //         test_assert!(BTreeSet<T>);
    //     }
    //     #[cfg(feature = "std")]
    //     #[allow(dead_code)]
    //     fn compound_wrapper_requiring_fromber_hash_eq<'a, T: FromBer<'a> + std::hash::Hash + Eq>(
    //         _: T,
    //     ) {
    //         use std::collections::HashSet;
    //         test_assert!(HashSet<T>);
    //     }
    // }

    // // test traits that should require BerParser
    // #[allow(dead_code)]
    // fn compound_wrapper<'a, T: BerParser<'a>>(_: T) {
    //     test_assert!(Option<T>);

    //     test_assert!(Vec<T>, SequenceOf<T>);

    //     // TODO: test for custom error types
    //     type E<'a> = BerError<Input<'a>>;
    //     test_assert!(TaggedExplicit<T, E, 0>);
    //     test_assert!(TaggedValue<T, E, Explicit, {Class::Application as u8}, 0>);

    //     #[allow(dead_code)]
    //     fn compound_wrapper_requiring_berparser_tagged<'a, T: BerParser<'a> + Tagged>(_: T) {
    //         // TODO: test for custom error types
    //         type E<'a> = BerError<Input<'a>>;
    //         test_assert!(TaggedImplicit<T, E, 0>);
    //         test_assert!(TaggedValue<T, E, Implicit, {Class::Application as u8}, 0>);
    //     }
    // }
}
