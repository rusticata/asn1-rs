use crate::{BerError, Input};

macro_rules! impl_parser_for_tuple {
    ($($parser:ident),+ / $error:ty) => {
        impl<$($parser),+,> $crate::Tagged for ($($parser),+,) {
            const CONSTRUCTED: bool = true;
            const TAG: crate::Tag = crate::Tag::Sequence;
        }

        #[allow(non_snake_case)]
        impl<'i, $($parser),+> $crate::BerParser<'i> for ($($parser),+,)
        where
            $($parser: $crate::BerParser<'i>),+,
            $($error: From<$parser::Error>),+,
        {
            type Error = $error;

            fn from_ber_content(
                _header: &'_ crate::Header<'i>,
                input: $crate::Input<'i>,
            ) -> nom::IResult<$crate::Input<'i>, Self, Self::Error> {
                let rem = input;
                $(let (rem, $parser) = <$parser>::parse_ber(rem).map_err(nom::Err::convert)?;)*
                Ok((rem, ($($parser),*,)))
            }
        }


        #[allow(non_snake_case)]
        impl<'i, $($parser),+> $crate::DerParser<'i> for ($($parser),+,)
        where
            $($parser: $crate::DerParser<'i>),+,
            $($error: From<$parser::Error>),+,
        {
            type Error = $error;

            fn from_der_content(
                _header: &'_ crate::Header<'i>,
                input: $crate::Input<'i>,
            ) -> nom::IResult<$crate::Input<'i>, Self, Self::Error> {
                let rem = input;
                $(let (rem, $parser) = <$parser>::parse_der(rem).map_err(nom::Err::convert)?;)*
                Ok((rem, ($($parser),*,)))
            }
        }
    };
}

macro_rules! impl_parser_for_tuples {
    ($parser1:ident, $($parser:ident),+ / $error:ty) => {
        impl_parser_for_tuples!(__impl $parser1; $($parser),+ / $error);
    };
    (__impl $($parser:ident),+; $parser1:ident $(,$parser2:ident)* / $error:ty) => {
        impl_parser_for_tuple!($($parser),+ / $error);
        impl_parser_for_tuples!(__impl $($parser),+, $parser1; $($parser2),* / $error);
    };
    (__impl $($parser:ident),+; / $error:ty) => {
        impl_parser_for_tuple!($($parser),+  / $error);
    }
}

// Implement BerParser for all tuples (T1, [T2, [...]]) where BerError<Input>>: From<Tn::Error>
// NOTE: we can only implement for a concrete error type. If using generic type E here,
// compiler will complain that E is unconstrained
impl_parser_for_tuples!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12 / BerError<Input<'i>>);

#[cfg(test)]
mod tests {
    use crate::{BerError, BerParser, DerParser, Input};

    #[test]
    fn assert_traits_tuples() {
        fn assert_berparser<'a, T: BerParser<'a>>() {}
        fn assert_derparser<'a, T: DerParser<'a>>() {}

        // test traits that should require BerParser
        #[allow(dead_code)]
        fn compound_berparser<'a, T>(_: T)
        where
            T: BerParser<'a>,
            // <T as BerParser<'a>>::Error: From<BerError<Input<'a>>>,
            BerError<Input<'a>>: From<<T as BerParser<'a>>::Error>,
        {
            assert_berparser::<(T,)>();
            assert_berparser::<(T, u32)>();
            assert_berparser::<(T, &str)>();
            assert_berparser::<(T, u32, &str)>();
        }

        // test traits that should require DerParser
        #[allow(dead_code)]
        fn compound_derparser<'a, T>(_: T)
        where
            T: DerParser<'a>,
            BerError<Input<'a>>: From<<T as DerParser<'a>>::Error>,
        {
            assert_derparser::<(T,)>();
            assert_derparser::<(T, u32)>();
            assert_derparser::<(T, &str)>();
            assert_derparser::<(T, u32, &str)>();
        }
    }
}
