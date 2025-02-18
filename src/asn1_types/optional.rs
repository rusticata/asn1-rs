use nom::bytes::streaming::take;
use nom::error::ParseError;

use crate::*;

// note: we cannot implement `TryFrom<Any<'a>> with generic errors for Option<T>`,
// because this conflicts with generic `T` implementation in
// `src/traits.rs`, since `T` always satisfies `T: Into<Option<T>>`
//
// for the same reason, we cannot use a generic error type here
impl<'a, T> FromBer<'a> for Option<T>
where
    T: FromBer<'a>,
    T: Tagged,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        if let Ok((_, header)) = Header::from_ber(bytes) {
            if T::TAG != header.tag {
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

impl<'a> FromBer<'a> for Option<Any<'a>> {
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        match Any::from_ber(bytes) {
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

    fn check_tag(_tag: Tag) -> bool {
        true
    }

    fn parse_ber(input: Input<'a>) -> IResult<Input<'a>, Self, Self::Error> {
        if input.is_empty() {
            return Ok((input, None));
        }
        // FIXME: call default trait impl
        // FIXME: default trait impl does not work: bytes are consumed even
        // if tag does not match

        let (rem, header) = Header::parse_ber(input.clone()).map_err(Err::convert)?;
        // TODO: handle indefinite
        let length = header
            .length
            .definite_inner()
            .map_err(BerError::convert_into(input.clone()))?;
        // FIXME: this will not check anything, Option<T>::check_tag always return true
        if !Self::check_tag(header.tag) {
            return Err(Err::Error(
                BerError::unexpected_tag(input, None, header.tag).into(),
            ));
        }
        // NOTE: we add this to default trait impl
        if !T::check_tag(header.tag) {
            return Ok((input, None));
        }
        // NOTE: end
        let (rem, data) = take(length)(rem)?;
        let (_, obj) = Self::from_any_ber(data, header).map_err(Err::convert)?;
        Ok((rem, obj))
    }

    fn from_any_ber(input: Input<'a>, header: Header<'a>) -> IResult<Input<'a>, Self, Self::Error> {
        if input.is_empty() {
            return Ok((input, None));
        }
        if !T::check_tag(header.tag) {
            return Ok((input, None));
        }
        // TODO: handle indefinite
        let length = header
            .length
            .definite_inner()
            .map_err(BerError::convert_into(input.clone()))?;
        let (rem, data) = take(length)(input)?;
        let (_, obj) = T::from_any_ber(data, header).map_err(Err::convert)?;
        Ok((rem, Some(obj)))
    }
}

impl<'a, T> FromDer<'a> for Option<T>
where
    T: FromDer<'a>,
    T: Tagged,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        if let Ok((_, header)) = Header::from_der(bytes) {
            if T::TAG != header.tag {
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

impl<'a> FromDer<'a> for Option<Any<'a>> {
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        match Any::from_der(bytes) {
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
}

#[cfg(feature = "std")]
impl<T> ToDer for Option<T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        match self {
            None => Ok(0),
            Some(t) => t.to_der_len(),
        }
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        match self {
            None => Ok(0),
            Some(t) => t.write_der_header(writer),
        }
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        match self {
            None => Ok(0),
            Some(t) => t.write_der_content(writer),
        }
    }
}

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
