use alloc::vec::Vec;
use nom::error::ParseError;
use nom::Err;

use crate::{BerError, BerParser, DerParser, InnerError, Input, Tag, Tagged};

use core::convert::TryFrom;

impl<T, const N: usize> Tagged for [T; N] {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::Sequence;
}

impl<'i, T, E, const N: usize> BerParser<'i> for [T; N]
where
    T: BerParser<'i, Error = E>,
    E: ParseError<Input<'i>> + From<BerError<Input<'i>>>,
{
    type Error = E;

    fn from_ber_content(
        header: &'_ crate::Header<'i>,
        input: Input<'i>,
    ) -> nom::IResult<Input<'i>, Self, Self::Error> {
        let (rem, v) = <Vec<T>>::from_ber_content(header, input.clone())?;
        let array = Self::try_from(v)
            .map_err(|_| Err::Error(BerError::new(input, InnerError::InvalidLength).into()))?;
        Ok((rem, array))
    }
}

impl<'i, T, E, const N: usize> DerParser<'i> for [T; N]
where
    T: DerParser<'i, Error = E>,
    E: ParseError<Input<'i>> + From<BerError<Input<'i>>>,
{
    type Error = E;

    fn from_der_content(
        header: &'_ crate::Header<'i>,
        input: Input<'i>,
    ) -> nom::IResult<Input<'i>, Self, Self::Error> {
        let (rem, v) = <Vec<T>>::from_der_content(header, input.clone())?;
        let array = Self::try_from(v)
            .map_err(|_| Err::Error(BerError::new(input, InnerError::InvalidLength).into()))?;
        Ok((rem, array))
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    use crate::{
        ber_length_constructed_items, der_length_constructed_items, Class, Constructed, Length,
        SerializeResult, ToBer, ToDer,
    };

    impl<T, const N: usize> ToBer for [T; N]
    where
        T: ToBer,
    {
        type Encoder = Constructed;

        fn ber_content_len(&self) -> Length {
            ber_length_constructed_items(self.iter())
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.iter().try_fold(0, |acc, t| {
                let sz = t.ber_encode(target)?;
                Ok(acc + sz)
            })
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }

    impl<T, const N: usize> ToDer for [T; N]
    where
        T: ToDer,
    {
        type Encoder = Constructed;

        fn der_content_len(&self) -> Length {
            der_length_constructed_items(self.iter())
        }

        fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.iter().try_fold(0, |acc, t| {
                let sz = t.der_encode(target)?;
                Ok(acc + sz)
            })
        }

        fn der_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, DerParser, Input};

    #[test]
    fn parse_ber_array_n() {
        type T = [u32; 2];

        // Ok: definite length
        let input = &hex!("30 0a 0203010001 0203010001");
        let (rem, res) = T::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(res[0], 65537);
        assert_eq!(res[1], 65537);

        // Ok: indefinite length
        let input = &hex!("30 80 0203010001 0203010001 0000");
        let (rem, res) = T::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(res[0], 65537);
        assert_eq!(res[1], 65537);

        // Fail: wrong length
        let input = &hex!("30 05 0203010001");
        let _ = T::parse_ber(Input::from(input)).expect_err("array / wrong length");
    }

    #[test]
    fn parse_der_array_n() {
        type T = [u32; 2];

        // Ok: definite length
        let input = &hex!("30 0a 0203010001 0203010001");
        let (rem, res) = T::parse_der(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(res[0], 65537);
        assert_eq!(res[1], 65537);

        // Ok: indefinite length
        let input = &hex!("30 80 0203010001 0203010001 0000");
        let _ = T::parse_der(Input::from(input)).expect_err("array / indefinite length");

        // Fail: wrong length
        let input = &hex!("30 05 0203010001");
        let _ = T::parse_der(Input::from(input)).expect_err("array / wrong length");
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{Integer, ToBer};

        #[test]
        fn tober_array_n() {
            let array = [Integer::from(1), Integer::from(65537)];
            let mut v: Vec<u8> = Vec::new();
            array.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(
                &v,
                &hex! {"30 08
                0201 01
                0203 010001"}
            );
        }
    }
}
