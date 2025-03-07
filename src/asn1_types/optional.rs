use nom::error::ParseError;

use crate::ber::{GetObjectContent, MAX_RECURSION};
use crate::*;

// note: we cannot implement `TryFrom<Any<'a>> with generic errors for Option<T>`,
// because this conflicts with generic `T` implementation in
// `src/traits.rs`, since `T` always satisfies `T: Into<Option<T>>`
//
// for the same reason, we cannot use a generic error type here
impl<'a, T> FromBer<'a> for Option<T>
where
    T: FromBer<'a>,
    T: DynTagged,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        if let Ok((_, header)) = Header::from_ber(bytes) {
            if !T::accept_tag(header.tag) {
                // not the expected tag, early return
                return Ok((bytes, None));
            }
        }
        match T::from_ber(bytes) {
            Ok((rem, t)) => Ok((rem, Some(t))),
            Err(e) => Err(e),
        }
    }
}

impl<'a, T, E> BerParser<'a> for Option<T>
where
    T: BerParser<'a, Error = E>,
    E: ParseError<Input<'a>> + From<BerError<Input<'a>>>,
{
    type Error = E;

    fn parse_ber(input: Input<'a>) -> IResult<Input<'a>, Self, Self::Error> {
        if input.is_empty() {
            return Ok((input, None));
        }
        // FIXME: call default trait impl?
        // FIXME: default trait impl does not work: bytes are consumed even
        // if tag does not match

        let (rem, header) = Header::parse_ber(input.clone()).map_err(Err::convert)?;
        // NOTE: we add this to default trait impl
        if !T::accept_tag(header.tag) {
            return Ok((input, None));
        }
        // NOTE: end
        let (rem, data) =
            BerMode::get_object_content(&header, rem, MAX_RECURSION).map_err(Err::convert)?;
        let (_, obj) = Self::from_ber_content(&header, data).map_err(Err::convert)?;
        Ok((rem, obj))
    }

    fn from_ber_content(
        header: &'_ Header<'a>,
        input: Input<'a>,
    ) -> IResult<Input<'a>, Self, Self::Error> {
        if input.is_empty() {
            return Ok((input, None));
        }
        if !T::accept_tag(header.tag) {
            return Ok((input, None));
        }
        let (rem, data) =
            BerMode::get_object_content(header, input, MAX_RECURSION).map_err(Err::convert)?;
        let (_, obj) = T::from_ber_content(header, data).map_err(Err::convert)?;
        Ok((rem, Some(obj)))
    }
}

impl<'a, T, E> DerParser<'a> for Option<T>
where
    T: DerParser<'a, Error = E>,
    E: ParseError<Input<'a>> + From<BerError<Input<'a>>>,
{
    type Error = E;

    fn parse_der(input: Input<'a>) -> IResult<Input<'a>, Self, Self::Error> {
        if input.is_empty() {
            return Ok((input, None));
        }
        // FIXME: call default trait impl?
        // FIXME: default trait impl does not work: bytes are consumed even
        // if tag does not match

        let (rem, header) = Header::parse_der(input.clone()).map_err(Err::convert)?;
        // NOTE: we add this to default trait impl
        if !T::accept_tag(header.tag) {
            return Ok((input, None));
        }
        // NOTE: end
        let (rem, data) =
            BerMode::get_object_content(&header, rem, MAX_RECURSION).map_err(Err::convert)?;
        let (_, obj) = Self::from_der_content(&header, data).map_err(Err::convert)?;
        Ok((rem, obj))
    }

    fn from_der_content(
        header: &'_ Header<'a>,
        input: Input<'a>,
    ) -> IResult<Input<'a>, Self, Self::Error> {
        if input.is_empty() {
            return Ok((input, None));
        }
        if !T::accept_tag(header.tag) {
            return Ok((input, None));
        }
        let (rem, data) =
            BerMode::get_object_content(header, input, MAX_RECURSION).map_err(Err::convert)?;
        let (_, obj) = T::from_der_content(header, data).map_err(Err::convert)?;
        Ok((rem, Some(obj)))
    }
}

impl<'a, T> FromDer<'a> for Option<T>
where
    T: FromDer<'a>,
    T: DynTagged,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        if let Ok((_, header)) = Header::from_der(bytes) {
            if !T::accept_tag(header.tag) {
                // not the expected tag, early return
                return Ok((bytes, None));
            }
        }
        match T::from_der(bytes) {
            Ok((rem, t)) => Ok((rem, Some(t))),
            Err(e) => Err(e),
        }
    }
}

impl<T> CheckDerConstraints for Option<T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        T::check_constraints(any)
    }
}

impl<T> DynTagged for Option<T>
where
    T: DynTagged,
{
    fn tag(&self) -> Tag {
        if self.is_some() {
            self.tag()
        } else {
            Tag(0)
        }
    }

    fn accept_tag(tag: Tag) -> bool {
        T::accept_tag(tag)
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl<T> ToBer for Option<T>
    where
        T: ToBer + DynTagged,
    {
        type Encoder = BerGenericEncoder;

        fn ber_content_len(&self) -> Length {
            match self {
                Some(t) => t.ber_content_len(),
                None => Length::Definite(0),
            }
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            match self {
                Some(t) => t.ber_write_content(target),
                None => Ok(0),
            }
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), self.constructed(), self.tag())
        }

        fn ber_encode<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            match self {
                Some(t) => t.ber_encode(target),
                None => Ok(0),
            }
        }
    }

    impl<T> ToDer for Option<T>
    where
        T: ToDer + DynTagged,
    {
        type Encoder = BerGenericEncoder;

        fn der_content_len(&self) -> Length {
            match self {
                Some(t) => t.der_content_len(),
                None => Length::Definite(0),
            }
        }

        fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            match self {
                Some(t) => t.der_write_content(target),
                None => Ok(0),
            }
        }

        fn der_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), self.constructed(), self.tag())
        }

        fn der_encode<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            match self {
                Some(t) => t.der_encode(target),
                None => Ok(0),
            }
        }
    }
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::BerParser;

    #[test]
    fn check_optional() {
        type T = Option<bool>;

        // parse correct value -> Ok
        let input: &[u8] = &hex! {"0101ff"};
        let (rem, res) = <T>::parse_ber(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res.is_some());

        // parse incorrect tag -> Ok (None)
        let input: &[u8] = &hex! {"020100"};
        let (rem, res) = <T>::parse_ber(input.into()).expect("parsing failed");
        // dbg!(&rem);
        // dbg!(&res);
        assert!(!rem.is_empty());
        assert!(res.is_none());
    }
}
