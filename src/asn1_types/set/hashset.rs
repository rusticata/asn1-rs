#![cfg(feature = "std")]
use crate::*;
use core::fmt::Debug;
use core::hash::BuildHasher;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::Hash;
use std::io::Write;

use self::debug::{trace, trace_generic};

impl<T, S> Tagged for HashSet<T, S> {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::Set;
}

impl<'a, T, S> TryFrom<Any<'a>> for HashSet<T, S>
where
    T: FromBer<'a>,
    T: Hash + Eq,
    S: BuildHasher + Default,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        let items = SetIterator::<T, BerMode>::new(any.data.as_bytes2())
            .collect::<Result<HashSet<T, S>>>()?;
        Ok(items)
    }
}

impl<'a, T, S> BerParser<'a> for HashSet<T, S>
where
    T: BerParser<'a>,
    T: Hash + Eq,
    S: BuildHasher + Default,
{
    type Error = <T as BerParser<'a>>::Error;

    fn from_ber_content(
        header: &'_ Header<'a>,
        input: Input<'a>,
    ) -> IResult<Input<'a>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.12.1)
        header
            .assert_constructed_input(&input)
            .map_err(|e| Err::Error(e.into()))?;

        AnyIterator::<BerMode>::new(input).try_parse_collect::<Self, T>()
    }
}

impl<'a, T, S> DerParser<'a> for HashSet<T, S>
where
    T: DerParser<'a>,
    T: Hash + Eq,
    S: BuildHasher + Default,
{
    type Error = <T as DerParser<'a>>::Error;

    fn from_der_content(
        header: &'_ Header<'a>,
        input: Input<'a>,
    ) -> IResult<Input<'a>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.12.1)
        header
            .assert_constructed_input(&input)
            .map_err(|e| Err::Error(e.into()))?;

        AnyIterator::<DerMode>::new(input).try_parse_collect::<Self, T>()
    }
}

impl<T, S> CheckDerConstraints for HashSet<T, S>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        for item in SetIterator::<Any, DerMode>::new(any.data.as_bytes2()) {
            let item = item?;
            T::check_constraints(&item)?;
        }
        Ok(())
    }
}

/// manual impl of FromDer, so we do not need to require `TryFrom<Any> + CheckDerConstraints`
impl<'a, T, E, S> FromDer<'a, E> for HashSet<T, S>
where
    T: FromDer<'a, E>,
    T: Hash + Eq,
    E: From<Error> + Debug,
    S: BuildHasher + Default,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        trace_generic(
            core::any::type_name::<Self>(),
            "BTreeSet::from_der",
            |bytes| {
                let (rem, any) = trace(core::any::type_name::<Self>(), Any::from_der, bytes)
                    .map_err(Err::convert)?;
                any.tag()
                    .assert_eq(Self::TAG)
                    .map_err(|e| Err::Error(e.into()))?;
                any.header
                    .assert_constructed()
                    .map_err(|e| Err::Error(e.into()))?;
                let items = SetIterator::<T, DerMode, E>::new(any.data.as_bytes2())
                    .collect::<Result<HashSet<T, S>, E>>()
                    .map_err(Err::Error)?;
                Ok((rem, items))
            },
            bytes,
        )
    }
}

impl<T, S> ToBer for HashSet<T, S>
where
    T: ToBer + DynTagged,
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

impl<T, S> ToDer for HashSet<T, S>
where
    T: ToDer + DynTagged,
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

#[cfg(test)]
mod tests {
    use crate::*;
    use core::convert::TryFrom;
    use hex_literal::hex;
    use std::collections::HashSet;

    #[test]
    fn ber_hashset() {
        let input = &hex! {"31 06 02 01 00 02 01 01"};
        let (_, any) = Any::from_ber(input).expect("parsing hashset failed");
        <HashSet<u32>>::check_constraints(&any).unwrap();

        let h = <HashSet<u32>>::try_from(any).unwrap();

        assert_eq!(h.len(), 2);
    }

    #[test]
    fn der_hashset() {
        let input = &hex! {"31 06 02 01 00 02 01 01"};
        let r: IResult<_, _, Error> = HashSet::<u32>::from_der(input);
        let (_, h) = r.expect("parsing hashset failed");

        assert_eq!(h.len(), 2);

        assert_eq!(h.der_content_len(), Length::Definite(6));
        let v = h.to_der_vec().expect("could not serialize");
        let (_, h2) = SetOf::<u32>::from_der(&v).unwrap();
        assert!(h.iter().eq(h2.iter()));
    }
}
