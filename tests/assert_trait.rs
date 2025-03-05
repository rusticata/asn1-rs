use asn1_rs::*;

fn assert_trait_ber_parser<'a, T: BerParser<'a>>() {}
fn assert_trait_der_parser<'a, T: DerParser<'a>>() {}
fn assert_trait_dyntagged<T: DynTagged>() {}

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

    test_assert!(OctetString, &[u8]);

    test_assert!(Real, f32, f64);

    test_assert!(Sequence, Set);
    test_assert!(AnySequence);

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

    // test traits that should require BerParser
    #[allow(dead_code)]
    fn compound_wrapper<'a, T: BerParser<'a>>(_: T) {
        test_assert!(Option<T>);

        test_assert!(Vec<T>, SequenceOf<T>);

        test_assert!(SetOf<T>);

        // TODO: test for custom error types
        type E<'a> = BerError<Input<'a>>;
        test_assert!(TaggedExplicit<T, E, 0>);
        test_assert!(TaggedValue<T, E, Explicit, {Class::Application as u8}, 0>);

        #[cfg(feature = "std")]
        #[allow(dead_code)]
        fn compound_wrapper_requiring_ord<'a, T: BerParser<'a> + Ord>(_: T) {
            use std::collections::BTreeSet;
            test_assert!(BTreeSet<T>);

            test_assert!(AnySet);
        }

        #[cfg(feature = "std")]
        #[allow(dead_code)]
        fn compound_wrapper_requiring_hash_eq<'a, T: BerParser<'a> + std::hash::Hash + Eq>(_: T) {
            use std::collections::HashSet;
            test_assert!(HashSet<T>);
        }

        #[allow(dead_code)]
        fn compound_wrapper_requiring_tagged<'a, T: BerParser<'a> + Tagged>(_: T) {
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

    test_assert!(BitString);
    test_assert!(Boolean, bool);
    test_assert!(Null, ());

    assert_trait_der_parser::<Enumerated>();

    assert_trait_der_parser::<Integer>();
    test_assert!(u8, u16, u32, u64, u128);
    test_assert!(i8, i16, i32, i64, i128);
    test_assert!(isize, usize);

    test_assert!(GeneralizedTime, UtcTime);

    test_assert!(OctetString, &[u8]);

    assert_trait_der_parser::<Oid>();

    test_assert!(Real, f32, f64);

    test_assert!(Sequence, Set);
    test_assert!(AnySequence);

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

    // test traits that should require BerParser
    #[allow(dead_code)]
    fn compound_wrapper<'a, T: DerParser<'a>>(_: T) {
        test_assert!(Option<T>);

        test_assert!(Vec<T>, SequenceOf<T>);

        test_assert!(SetOf<T>);

        #[cfg(feature = "std")]
        #[allow(dead_code)]
        fn compound_wrapper_requiring_ord<'a, T>(_: T)
        where
            T: DerParser<'a> + Ord,
        {
            use std::collections::BTreeSet;
            test_assert!(BTreeSet<T>);

            test_assert!(AnySet);
        }

        #[cfg(feature = "std")]
        #[allow(dead_code)]
        fn compound_wrapper_requiring_hash_eq<'a, T>(_: T)
        where
            T: DerParser<'a> + std::hash::Hash + Eq,
        {
            use std::collections::HashSet;
            test_assert!(HashSet<T>);
        }

        // TODO: test for custom error types
        type E<'a> = BerError<Input<'a>>;
        test_assert!(TaggedExplicit<T, E, 0>);
        test_assert!(TaggedValue<T, E, Explicit, {Class::Application as u8}, 0>);

        #[allow(dead_code)]
        fn compound_wrapper_requiring_berparser_tagged<'a, T: DerParser<'a> + Tagged>(_: T) {
            // TODO: test for custom error types
            type E<'a> = BerError<Input<'a>>;
            test_assert!(TaggedImplicit<T, E, 0>);
            test_assert!(TaggedValue<T, E, Implicit, {Class::Application as u8}, 0>);
        }
    }
}

/// Compile-time verification that all supported types implement FromDer
#[test]
fn assert_traits_dyntagged() {
    macro_rules! test_assert {
        ($($_type:ty),*) => {
            $( test_assert!(_SINGLE $_type); )*
        };

        (_SINGLE $_type:ty) => {
            assert_trait_dyntagged::<$_type>();
        };
    }

    test_assert!(Header, Any);

    test_assert!(BitString);
    test_assert!(Boolean, bool);
    test_assert!(Null, ());

    test_assert!(Enumerated);

    test_assert!(Integer);
    test_assert!(u8, u16, u32, u64, u128);
    test_assert!(i8, i16, i32, i64, i128);
    test_assert!(isize, usize);

    test_assert!(GeneralizedTime, UtcTime);

    test_assert!(OctetString, &[u8]);

    test_assert!(Oid);

    test_assert!(Real, f32, f64);

    test_assert!(Sequence, Set);
    test_assert!(AnySequence);

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

    // test traits that should provide DynTagged
    #[allow(dead_code)]
    fn compound_wrapper<T>(_: T) {
        test_assert!(Vec<T>, SequenceOf<T>);

        test_assert!(SetOf<T>);

        // TODO: test for custom error types
        type E<'a> = BerError<Input<'a>>;
        test_assert!(TaggedExplicit<T, E, 0>);
        test_assert!(TaggedValue<T, E, Explicit, {Class::Application as u8}, 0>);
    }

    // test traits that should provide DynTagged
    #[allow(dead_code)]
    fn compound_wrapper_dyntagged<T>(_: T)
    where
        T: DynTagged,
    {
        test_assert!(Option<T>);

        // TODO: test for custom error types
        type E<'a> = BerError<Input<'a>>;
        test_assert!(TaggedImplicit<T, E, 0>);
        test_assert!(TaggedValue<T, E, Implicit, {Class::Application as u8}, 0>);
    }

    #[cfg(feature = "std")]
    #[allow(dead_code)]
    fn compound_wrapper_std<T>(_: T) {
        use std::collections::{BTreeSet, HashSet};

        test_assert!(AnySet);

        test_assert!(HashSet<T>);
        test_assert!(BTreeSet<T>);
    }
}

#[cfg(feature = "std")]
#[test]
fn assert_traits_tober() {
    fn assert_trait_tober<T: ToBer>() {}

    macro_rules! test_assert {
        ($($_type:ty),*) => {
            $( test_assert!(_SINGLE $_type); )*
        };

        (_SINGLE $_type:ty) => {
            assert_trait_tober::<$_type>();
        };
    }

    test_assert!(Any);

    test_assert!(BitString);
    test_assert!(Boolean, bool);
    test_assert!(Null, ());

    test_assert!(Enumerated);

    test_assert!(Integer);
    test_assert!(u8, u16, u32, u64, u128);
    test_assert!(i8, i16, i32, i64, i128);
    test_assert!(isize, usize);

    test_assert!(GeneralizedTime, UtcTime);

    test_assert!(OctetString, &[u8]);

    test_assert!(Oid);

    test_assert!(Real, f32, f64);

    test_assert!(Sequence, Set);
    test_assert!(AnySequence);
    test_assert!(AnySet);

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

    #[allow(dead_code)]
    fn compound_wrapper_std<T>(_: T) {
        use std::collections::{BTreeSet, HashSet};

        // test_assert!(SequenceOf<T>);
        // test_assert!(SetOf<T>);

        #[allow(dead_code)]
        fn compound_wrapper_tober<T>(_: T)
        where
            T: ToBer + DynTagged,
        {
            test_assert!(HashSet<T>);
            test_assert!(BTreeSet<T>);
        }
    }
}
