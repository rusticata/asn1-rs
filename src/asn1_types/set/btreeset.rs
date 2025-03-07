use crate::*;
use alloc::collections::BTreeSet;
use core::{convert::TryFrom, fmt::Debug};

use self::debug::{trace, trace_generic};

impl<T> Tagged for BTreeSet<T> {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::Set;
}

impl<'a, T> TryFrom<Any<'a>> for BTreeSet<T>
where
    T: FromBer<'a>,
    T: Ord,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        trace_generic(
            core::any::type_name::<Self>(),
            "BTreeSet::from_der",
            |any| {
                any.tag().assert_eq(Self::TAG)?;
                any.header.assert_constructed()?;
                let items = SetIterator::<T, BerMode>::new(any.data.as_bytes2())
                    .collect::<Result<BTreeSet<T>>>()?;
                Ok(items)
            },
            any,
        )
    }
}

impl<'a, T> BerParser<'a> for BTreeSet<T>
where
    T: BerParser<'a>,
    T: Ord,
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

impl<'a, T> DerParser<'a> for BTreeSet<T>
where
    T: DerParser<'a>,
    T: Ord,
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

impl<T> CheckDerConstraints for BTreeSet<T>
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
impl<'a, T, E> FromDer<'a, E> for BTreeSet<T>
where
    T: FromDer<'a, E>,
    T: Ord,
    E: From<Error> + Debug,
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
                    .collect::<Result<BTreeSet<T>, E>>()
                    .map_err(Err::Error)?;
                Ok((rem, items))
            },
            bytes,
        )
    }
}

#[cfg(feature = "std")]
impl<T> ToDer for BTreeSet<T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        let mut len = 0;
        for t in self.iter() {
            len += t.to_der_len()?;
        }
        let header = Header::new(Class::Universal, true, Self::TAG, Length::Definite(len));
        Ok(header.to_der_len()? + len)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut len = 0;
        for t in self.iter() {
            len += t.to_der_len().map_err(|_| SerializeError::InvalidLength)?;
        }
        let header = Header::new(Class::Universal, true, Self::TAG, Length::Definite(len));
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut sz = 0;
        for t in self.iter() {
            sz += t.write_der(writer)?;
        }
        Ok(sz)
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;
    impl<T> ToBer for BTreeSet<T>
    where
        T: ToBer + DynTagged,
    {
        type Encoder = Constructed;

        fn content_len(&self) -> Length {
            // content_len returns only the length of *content*, so we need header length for
            // every object here
            let len = self.iter().fold(Length::Definite(0), |acc, t| {
                let content_length = t.content_len();
                match (acc, content_length) {
                    (Length::Definite(a), Length::Definite(b)) => {
                        let header_length = ber_header_length(t.tag(), content_length).unwrap_or(0);
                        Length::Definite(a + header_length + b)
                    }
                    _ => Length::Indefinite,
                }
            });
            len
        }

        fn write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.iter().try_fold(0, |acc, t| {
                let sz = t.encode(target)?;
                Ok(acc + sz)
            })
        }

        fn tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }
};

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::*;
    use core::convert::TryFrom;
    use hex_literal::hex;
    use std::collections::BTreeSet;

    #[test]
    fn ber_btreeset() {
        let input = &hex! {"31 06 02 01 00 02 01 01"};
        let (_, any) = Any::from_ber(input).expect("parsing hashset failed");
        <BTreeSet<u32>>::check_constraints(&any).unwrap();

        let h = <BTreeSet<u32>>::try_from(any).unwrap();

        assert_eq!(h.len(), 2);
    }

    #[test]
    fn der_btreeset() {
        let input = &hex! {"31 06 02 01 00 02 01 01"};
        let r: IResult<_, _, Error> = BTreeSet::<u32>::from_der(input);
        let (_, h) = r.expect("parsing hashset failed");

        assert_eq!(h.len(), 2);

        assert_eq!(h.to_der_len(), Ok(8));
        let v = h.to_der_vec().expect("could not serialize");
        let (_, h2) = SetOf::<u32>::from_der(&v).unwrap();
        assert!(h.iter().eq(h2.iter()));
    }
}
