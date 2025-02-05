use crate::debug::trace_generic;
use crate::*;
use core::fmt::Debug;

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

/// An `Option` type
///
/// This option type has been implemented so we can implement FromBer and FromDer
/// without conflicting with other implementations.
///
/// `TryFrom<Any<'a>> cannot be implemented with generic errors for Option<T>`,
/// because this conflicts with generic `T` implementation in
/// `src/traits.rs`, since `T` always satisfies `T: Into<Option<T>>`
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct BerOption<T>(Option<T>);

impl<T> BerOption<T> {
    /// Build a new `BerOption`
    #[inline]
    pub const fn new(t: Option<T>) -> Self {
        Self(t)
    }

    /// Returns `Option<&T>`
    pub const fn as_ref(&self) -> Option<&T> {
        self.0.as_ref()
    }

    /// Returns `Option<&mut T>`
    #[rustversion::attr(since(1.83), const)]
    pub fn as_mut(&mut self) -> Option<&mut T> {
        self.0.as_mut()
    }
}

impl<'a, T, E> FromBer<'a, E> for BerOption<T>
where
    T: FromBer<'a, E>,
    T: Tagged,
    E: Debug,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        trace_generic(
            core::any::type_name::<Self>(),
            "BerOption::from_ber",
            |bytes| {
                if bytes.is_empty() {
                    return Ok((bytes, BerOption(None)));
                }
                if let Ok((_, header)) = Header::from_ber(bytes) {
                    if T::TAG != header.tag {
                        // not the expected tag, early return
                        return Ok((bytes, BerOption(None)));
                    }
                }
                match T::from_ber(bytes) {
                    Ok((rem, t)) => Ok((rem, BerOption(Some(t)))),
                    Err(e) => Err(e),
                }
            },
            bytes,
        )
    }
}

impl<'a, T, E> FromDer<'a, E> for BerOption<T>
where
    T: FromDer<'a, E>,
    T: Tagged,
    E: Debug,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        trace_generic(
            core::any::type_name::<Self>(),
            "BerOption::from_der",
            |bytes| {
                if bytes.is_empty() {
                    return Ok((bytes, BerOption(None)));
                }
                if let Ok((_, header)) = Header::from_ber(bytes) {
                    if T::TAG != header.tag {
                        // not the expected tag, early return
                        return Ok((bytes, BerOption(None)));
                    }
                }
                match T::from_der(bytes) {
                    Ok((rem, t)) => Ok((rem, BerOption(Some(t)))),
                    Err(e) => Err(e),
                }
            },
            bytes,
        )
    }
}

impl<T> CheckDerConstraints for BerOption<T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        T::check_constraints(any)
    }
}

impl<T> DerAutoDerive for BerOption<T> {}

impl<T> Tagged for BerOption<T>
where
    T: Tagged,
{
    const TAG: Tag = T::TAG;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    pub struct CustomError {}

    impl From<Error> for CustomError {
        fn from(_value: Error) -> Self {
            CustomError {}
        }
    }

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
    fn test_option_parser_inference_opt() {
        let data: &[u8] = &[0x01];
        // // will not work: custom error
        // let res = <Option<MyType>>::from_der(data);
        let res = <BerOption<MyType>>::from_der(data);
        assert!(matches!(res, Err(Err::Error(CustomError {}))));
    }

    #[test]
    fn test_option_parser_inference_opt_vec() {
        let data: &[u8] = &[0x30, 0x02, 0x30, 0x01];
        // // will not work: custom error
        // let res = <Vec<Option<MyType>>>::from_der(data);
        let res = <BerOption<Vec<MyType>>>::from_der(data);
        assert!(matches!(res, Err(Err::Error(CustomError {}))));
    }
}
