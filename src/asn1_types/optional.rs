use crate::*;

// note: we cannot implement `TryFrom<Any<'a>> with generic errors for Option<T>`,
// because this conflicts with generic `T` implementation in
// `src/traits.rs`, since `T` always satisfies `T: Into<Option<T>>`
//
// for the same reason, we cannot use a generic error type here
impl<'a, T, E> FromBer<'a, E> for Option<T>
where
    T: FromBer<'a, E>,
    T: Tagged,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
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

impl<'a, T, E> FromDer<'a, E> for Option<T>
where
    T: FromDer<'a, E>,
    T: Tagged,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
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
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    pub struct CustomError {}

    #[derive(Debug)]
    pub struct MyType {
        _a: u32,
    }

    impl<'a> FromDer<'a, CustomError> for MyType {
        fn from_der(_bytes: &'a [u8]) -> ParseResult<'a, Self, CustomError> {
            Err(Err::Error(CustomError {}))
        }
    }
    impl Tagged for MyType {
        const TAG: Tag = Tag::Sequence;
    }

    /// test if we are able to define & build code with option and custom error
    #[test]
    fn test_option_parser_inference() {
        let data = &[0x01];
        let res = <Option<MyType>>::from_der(data);
        assert!(matches!(res, Err(Err::Error(CustomError {}))));
    }
}
