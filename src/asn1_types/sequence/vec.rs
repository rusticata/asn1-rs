use crate::*;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::fmt::Debug;

use self::debug::{trace, trace_generic};

// // XXX this compiles but requires bound TryFrom :/
// impl<'a, 'b, T> TryFrom<&'b Any<'a>> for Vec<T>
// where
//     T: TryFrom<&'b Any<'a>>,
//     for<'e> <T as TryFrom<&'b Any<'a>>>::Error: From<Error>,
//     T: FromBer<'a, <T as TryFrom<&'b Any<'a>>>::Error>,
//     //     T: FromBer<'a, E>,
//     //     E: From<Error>,
// {
//     type Error = <T as TryFrom<&'b Any<'a>>>::Error;

//     fn try_from(any: &'b Any<'a>) -> Result<Vec<T>, Self::Error> {
//         any.tag().assert_eq(Self::TAG)?;
//         any.header.assert_constructed()?;
//         let v = SequenceIterator::<T, BerParser, Self::Error>::new(any.data)
//             .collect::<Result<Vec<T>, Self::Error>>()?;
//         Ok(v)
//     }
// }

// // XXX this compiles but requires bound TryFrom :/
// impl<'a, 'b, T> TryFrom<&'b Any<'a>> for Vec<T>
// where
//     T: TryFrom<&'b Any<'a>>,
//     <T as TryFrom<&'b Any<'a>>>::Error: From<Error>,
//     T: FromBer<'a, <T as TryFrom<&'b Any<'a>>>::Error>,
//     //     T: FromBer<'a, E>,
//     //     E: From<Error>,
// {
//     type Error = <T as TryFrom<&'b Any<'a>>>::Error;

//     fn try_from(any: &'b Any<'a>) -> Result<Vec<T>, Self::Error> {
//         any.tag().assert_eq(Self::TAG)?;
//         any.header.assert_constructed()?;
//         let v = SequenceIterator::<T, BerParser, Self::Error>::new(any.data)
//             .collect::<Result<Vec<T>, Self::Error>>()?;
//         Ok(v)
//     }
// }

impl<'a, T> TryFrom<Any<'a>> for Vec<T>
where
    T: FromBer<'a>,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;

        SetIterator::<T, BerMode>::new(any.data.as_bytes2()).collect()
    }
}

impl<'i, T> BerParser<'i> for Vec<T>
where
    T: BerParser<'i>,
    <T as BerParser<'i>>::Error: From<BerError<Input<'i>>>,
{
    type Error = <T as BerParser<'i>>::Error;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.10.1)
        header
            .assert_constructed_input(&input)
            .map_err(|e| Err::Error(e.into()))?;

        // NOTE: we cannot use many0 here, it silently converts Error to Ok
        // let (rem, v) = many0(cut(T::parse_ber)).parse(input)?;

        if input.is_empty() {
            return Ok((input, Vec::new()));
        }

        AnyIterator::<BerMode>::new(input).try_parse_collect::<Vec<_>, T>()
    }
}

impl<'i, T> DerParser<'i> for Vec<T>
where
    T: DerParser<'i>,
    <T as DerParser<'i>>::Error: From<BerError<Input<'i>>>,
{
    type Error = <T as DerParser<'i>>::Error;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.10.1)
        header
            .assert_constructed_input(&input)
            .map_err(|e| Err::Error(e.into()))?;

        // NOTE: we cannot use many0 here, it silently converts Error to Ok
        // let (rem, v) = many0(cut(T::parse_ber)).parse(input)?;

        if input.is_empty() {
            return Ok((input, Vec::new()));
        }

        AnyIterator::<DerMode>::new(input).try_parse_collect::<Vec<_>, T>()
    }
}

impl<T> CheckDerConstraints for Vec<T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        for item in SequenceIterator::<Any, DerMode>::new(any.data.as_bytes2()) {
            let item = item?;
            <T as CheckDerConstraints>::check_constraints(&item)?;
        }
        Ok(())
    }
}

impl<T> Tagged for Vec<T> {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::Sequence;
}

// impl<'a, T> FromBer<'a> for Vec<T>
// where
//     T: FromBer<'a>,
// {
//     fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
//         let (rem, any) = Any::from_ber(bytes)?;
//         any.header.assert_tag(Self::TAG)?;
//         let v = SequenceIterator::<T, BerParser>::new(any.data).collect::<Result<Vec<T>>>()?;
//         Ok((rem, v))
//     }
// }

/// manual impl of FromDer, so we do not need to require `TryFrom<Any> + CheckDerConstraints`
impl<'a, T, E> FromDer<'a, E> for Vec<T>
where
    T: FromDer<'a, E>,
    E: From<Error> + Debug,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        trace_generic(
            core::any::type_name::<Self>(),
            "Sequence::from_der",
            |bytes| {
                let (rem, any) = trace(
                    core::any::type_name::<Self>(),
                    wrap_ber_parser(parse_der_any),
                    bytes,
                )
                .map_err(Err::convert)?;
                any.header
                    .assert_tag(Self::TAG)
                    .map_err(|e| Err::Error(e.into()))?;
                let v = SequenceIterator::<T, DerMode, E>::new(any.data.as_bytes2())
                    .collect::<Result<Vec<T>, E>>()
                    .map_err(Err::Error)?;
                Ok((rem, v))
            },
            bytes,
        )
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    // NOTE: we need T::DynTagged (T can be a CHOICE)
    impl<T> ToBer for Vec<T>
    where
        T: ToBer,
        T: DynTagged,
        // Vec<T>: DynTagged,
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

    impl<T> ToDer for Vec<T>
    where
        T: ToDer,
        T: DynTagged,
        // Vec<T>: DynTagged,
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
