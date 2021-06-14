use crate::{Any, CheckDerConstraints, Error, FromBer, FromDer, ParseResult};

impl<'a, T> FromBer<'a> for Option<T>
where
    T: FromBer<'a>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        match T::from_ber(bytes) {
            Ok((rem, t)) => Ok((rem, Some(t))),
            Err(nom::Err::Failure(Error::UnexpectedTag(_))) => Ok((bytes, None)),
            Err(e) => Err(e),
        }
    }
}

impl<'a, T> FromDer<'a> for Option<T>
where
    T: FromDer<'a>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<Self> {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        match T::from_der(bytes) {
            Ok((rem, t)) => Ok((rem, Some(t))),
            Err(nom::Err::Failure(Error::UnexpectedTag(_))) => Ok((bytes, None)),
            Err(e) => Err(e),
        }
    }
}

impl<T> CheckDerConstraints for Option<T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> crate::Result<()> {
        T::check_constraints(any)
    }
}
