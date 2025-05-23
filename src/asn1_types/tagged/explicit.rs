use crate::*;
use core::convert::TryFrom;
use core::marker::PhantomData;

impl<T, E, const CLASS: u8, const TAG: u32> Tagged for TaggedValue<T, E, Explicit, CLASS, TAG> {
    const CLASS: Class = Class::new_unwrap(CLASS);
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag(TAG);
}

impl<'a, T, E, const CLASS: u8, const TAG: u32> TryFrom<Any<'a>>
    for TaggedValue<T, E, Explicit, CLASS, TAG>
where
    T: FromBer<'a, E>,
    E: From<Error>,
{
    type Error = E;

    fn try_from(any: Any<'a>) -> Result<Self, E> {
        Self::try_from(&any)
    }
}

impl<'a, 'b, T, E, const CLASS: u8, const TAG: u32> TryFrom<&'b Any<'a>>
    for TaggedValue<T, E, Explicit, CLASS, TAG>
where
    T: FromBer<'a, E>,
    E: From<Error>,
{
    type Error = E;

    fn try_from(any: &'b Any<'a>) -> Result<Self, E> {
        any.tag().assert_eq(Tag(TAG))?;
        any.header.assert_constructed()?;
        if any.class() as u8 != CLASS {
            let class = Class::try_from(CLASS).ok();
            return Err(Error::unexpected_class(class, any.class()).into());
        }
        let (_, inner) = match T::from_ber(any.data.as_bytes2()) {
            Ok((rem, res)) => (rem, res),
            Err(Err::Error(e)) | Err(Err::Failure(e)) => return Err(e),
            Err(Err::Incomplete(n)) => return Err(Error::Incomplete(n).into()),
        };
        Ok(TaggedValue::explicit(inner))
    }
}

impl<'i, T, E, const CLASS: u8, const TAG: u32> BerParser<'i>
    for TaggedValue<T, E, Explicit, CLASS, TAG>
where
    // T: Tagged,
    T: BerParser<'i>,
    // E: ParseError<Input<'a>> + From<BerError<Input<'a>>>,
{
    type Error = T::Error;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Tagged Explicit must be constructed (X.690 8.14.2)
        header
            .assert_constructed_input(&input)
            .map_err(|e| Err::Error(e.into()))?;

        // assert class
        if header.class as u8 != CLASS {
            // Safety: CLASS < 4
            let class = Class::try_from(CLASS).unwrap_or(Class::Private);
            return Err(Err::Error(
                BerError::unexpected_class(input, Some(class), header.class).into(),
            ));
        }

        // note: we check tag here, because the only way to have a different tag
        // would be to be IMPLICIT, and we already know we are EXPLICIT
        // This is an exception!
        if !Self::accept_tag(header.tag) {
            return Err(Err::Error(
                BerError::unexpected_tag(input, Some(TAG.into()), header.tag).into(),
            ));
        }
        // calling `parse_ber` will read a new header and parse object `T``
        // this will also check that T::TAG is expected
        let (rem, t) = T::parse_ber(input)?;
        let tagged = TaggedValue::explicit(t);
        Ok((rem, tagged))
    }
}

impl<'i, T, E, const CLASS: u8, const TAG: u32> DerParser<'i>
    for TaggedValue<T, E, Explicit, CLASS, TAG>
where
    // T: Tagged,
    T: DerParser<'i>,
    // E: ParseError<Input<'a>> + From<BerError<Input<'a>>>,
{
    type Error = T::Error;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Tagged Explicit must be constructed (X.690 8.14.2)
        header
            .assert_constructed_input(&input)
            .map_err(|e| Err::Error(e.into()))?;

        // assert class
        if header.class as u8 != CLASS {
            // Safety: CLASS < 4
            let class = Class::try_from(CLASS).unwrap_or(Class::Private);
            return Err(Err::Error(
                BerError::unexpected_class(input, Some(class), header.class).into(),
            ));
        }

        // note: we check tag here, because the only way to have a different tag
        // would be to be IMPLICIT, and we already know we are EXPLICIT
        // This is an exception!
        if !Self::accept_tag(header.tag) {
            return Err(Err::Error(
                BerError::unexpected_tag(input, Some(TAG.into()), header.tag).into(),
            ));
        }
        // calling `parse_ber` will read a new header and parse object `T``
        // this will also check that T::TAG is expected
        let (rem, t) = T::parse_der(input)?;
        let tagged = TaggedValue::explicit(t);
        Ok((rem, tagged))
    }
}

impl<'a, T, E, const CLASS: u8, const TAG: u32> FromDer<'a, E>
    for TaggedValue<T, E, Explicit, CLASS, TAG>
where
    T: FromDer<'a, E>,
    E: From<Error>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        let (rem, any) = Any::from_der(bytes).map_err(Err::convert)?;
        any.tag()
            .assert_eq(Tag(TAG))
            .map_err(|e| Err::Error(e.into()))?;
        any.header
            .assert_constructed()
            .map_err(|e| Err::Error(e.into()))?;
        if any.class() as u8 != CLASS {
            let class = Class::try_from(CLASS).ok();
            return Err(Err::Error(
                Error::unexpected_class(class, any.class()).into(),
            ));
        }
        let (_, inner) = T::from_der(any.data.as_bytes2())?;
        Ok((rem, TaggedValue::explicit(inner)))
    }
}

impl<T, E, const CLASS: u8, const TAG: u32> CheckDerConstraints
    for TaggedValue<T, E, Explicit, CLASS, TAG>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let (_, inner) = Any::from_ber(any.data.as_bytes2())?;
        T::check_constraints(&inner)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
const _: () = {
    impl<T, E, const CLASS: u8, const TAG: u32> ToBer for TaggedValue<T, E, Explicit, CLASS, TAG>
    where
        T: ToBer,
        T: DynTagged,
    {
        type Encoder = BerGenericEncoder;

        fn ber_content_len(&self) -> Length {
            let content_len = self.inner.ber_content_len();
            let header_len = ber_header_length(self.inner.tag(), content_len).unwrap_or(0);
            header_len + content_len
        }

        fn ber_write_content<W: std::io::Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.inner.ber_encode(target)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }

    impl<T, E, const CLASS: u8, const TAG: u32> ToDer for TaggedValue<T, E, Explicit, CLASS, TAG>
    where
        T: ToDer,
        T: DynTagged,
    {
        type Encoder = BerGenericEncoder;

        fn der_content_len(&self) -> Length {
            let content_len = self.inner.der_content_len();
            let header_len = ber_header_length(self.inner.tag(), content_len).unwrap_or(0);
            header_len + content_len
        }

        fn der_write_content<W: std::io::Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.inner.der_encode(target)
        }

        fn der_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }
};

/// A helper object to parse `[ n ] EXPLICIT T`
///
/// A helper object implementing [`FromBer`] and [`FromDer`], to parse tagged
/// optional values.
///
/// This helper expects context-specific tags.
/// See [`TaggedValue`] or [`TaggedParser`] for more generic implementations if needed.
///
/// # Examples
///
/// To parse a `[0] EXPLICIT INTEGER` object:
///
/// ```rust
/// use asn1_rs::{Error, FromBer, Integer, TaggedExplicit, TaggedValue};
///
/// let bytes = &[0xa0, 0x03, 0x2, 0x1, 0x2];
///
/// // If tagged object is present (and has expected tag), parsing succeeds:
/// let (_, tagged) = TaggedExplicit::<Integer, Error, 0>::from_ber(bytes).unwrap();
/// assert_eq!(tagged, TaggedValue::explicit(Integer::from(2)));
/// ```
pub type TaggedExplicit<T, E, const TAG: u32> = TaggedValue<T, E, Explicit, CONTEXT_SPECIFIC, TAG>;

// implementations for TaggedParser

impl<'a, T, E> TaggedParser<'a, Explicit, T, E> {
    pub const fn new_explicit(class: Class, tag: u32, inner: T) -> Self {
        Self {
            header: Header::new(class, true, Tag(tag), Length::Definite(0)),
            inner,
            tag_kind: PhantomData,
            _e: PhantomData,
        }
    }

    /// Parse a BER tagged value and apply the provided parsing function to content
    ///
    /// After parsing, the sequence object and header are discarded.
    ///
    /// Note: this function is provided for `Explicit`, but there is not difference between
    /// explicit or implicit tags. The `op` function is responsible of handling the content.
    #[inline]
    pub fn from_ber_and_then<F>(
        class: Class,
        tag: u32,
        bytes: &'a [u8],
        op: F,
    ) -> ParseResult<'a, T, E>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<'a, T, E>,
        E: From<Error>,
    {
        Any::from_ber_and_then(class, tag, bytes, op)
    }

    /// Parse a DER tagged value and apply the provided parsing function to content
    ///
    /// After parsing, the sequence object and header are discarded.
    ///
    /// Note: this function is provided for `Explicit`, but there is not difference between
    /// explicit or implicit tags. The `op` function is responsible of handling the content.
    #[inline]
    pub fn from_der_and_then<F>(
        class: Class,
        tag: u32,
        bytes: &'a [u8],
        op: F,
    ) -> ParseResult<'a, T, E>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<'a, T, E>,
        E: From<Error>,
    {
        Any::from_der_and_then(class, tag, bytes, op)
    }
}

impl<'a, T, E> FromBer<'a, E> for TaggedParser<'a, Explicit, T, E>
where
    T: FromBer<'a, E>,
    E: From<Error>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        let (rem, any) = Any::from_ber(bytes).map_err(Err::convert)?;
        let header = any.header;
        let (_, inner) = T::from_ber(any.data.as_bytes2())?;
        let tagged = TaggedParser {
            header,
            inner,
            tag_kind: PhantomData,
            _e: PhantomData,
        };
        Ok((rem, tagged))
    }
}

impl<'a, T, E> FromDer<'a, E> for TaggedParser<'a, Explicit, T, E>
where
    T: FromDer<'a, E>,
    E: From<Error>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        let (rem, any) = Any::from_der(bytes).map_err(Err::convert)?;
        let header = any.header;
        let (_, inner) = T::from_der(any.data.as_bytes2())?;
        let tagged = TaggedParser {
            header,
            inner,
            tag_kind: PhantomData,
            _e: PhantomData,
        };
        Ok((rem, tagged))
    }
}

impl<T> CheckDerConstraints for TaggedParser<'_, Explicit, T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let (_, inner_any) = Any::from_der(any.data.as_bytes2())?;
        T::check_constraints(&inner_any)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
const _: () = {
    impl<T, E> ToBer for TaggedParser<'_, Explicit, T, E>
    where
        T: ToBer,
    {
        type Encoder = BerGenericEncoder;

        fn ber_content_len(&self) -> Length {
            self.inner.ber_total_len()
        }

        fn ber_write_content<W: std::io::Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.inner.ber_encode(target)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), true, self.tag())
        }
    }

    impl<T, E> ToDer for TaggedParser<'_, Explicit, T, E>
    where
        T: ToDer,
    {
        type Encoder = BerGenericEncoder;

        fn der_content_len(&self) -> Length {
            self.inner.der_total_len()
        }

        fn der_write_content<W: std::io::Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.inner.der_encode(target)
        }

        fn der_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), true, self.tag())
        }
    }
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerError, BerParser, Input, TaggedExplicit};

    #[test]
    fn check_tagged_explicit() {
        type T<'a> = TaggedExplicit<bool, BerError<Input<'a>>, 0>;

        // untagged value -> should fail
        let input: &[u8] = &hex! {"0101ff"};
        let _res = T::parse_ber(input.into()).expect_err("parsing should have failed");

        // tagged value, correct tag -> Ok
        let input: &[u8] = &hex! {"a0 03 0101ff"};
        let _res = T::parse_ber(input.into()).expect("parsing failed");

        // tagged value, incorrect tag -> Fail
        let input: &[u8] = &hex! {"a1 03 0101ff"};
        let _res = T::parse_ber(input.into()).expect_err("parsing should have failed");

        // tagged value, correct tag but incorrect class -> Fail
        let input: &[u8] = &hex! {"60 03 0101ff"};
        let _res = T::parse_ber(input.into()).expect_err("parsing should have failed");
    }

    #[test]
    fn check_opttagged_explicit() {
        // **** using parse_ber_optional ****
        type T1<'a> = TaggedExplicit<bool, BerError<Input<'a>>, 0>;

        // empty -> OK, should return None
        let input: &[u8] = &hex! {""};
        let res = T1::parse_ber_optional(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // incorrect tag -> OK, should return None
        let input: &[u8] = &hex! {"a1 03 0101ff"};
        let res = T1::parse_ber_optional(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // tagged value, correct tag -> Ok
        let input: &[u8] = &hex! {"a0 03 0101ff"};
        let (rem, res) = T1::parse_ber_optional(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res.is_some());

        // tagged value, correct tag and invalid content -> Fail
        let input: &[u8] = &hex! {"a0 03 0102ffff"};
        let _res = T1::parse_ber_optional(input.into()).expect_err("parsing should have failed");

        // **** using Option<T> ****
        type T2<'a> = Option<TaggedExplicit<bool, BerError<Input<'a>>, 0>>;

        // empty -> OK, should return None
        let input: &[u8] = &hex! {""};
        let res = T2::parse_ber(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // incorrect tag -> OK, should return None
        let input: &[u8] = &hex! {"a1 03 0101ff"};
        let res = T2::parse_ber(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // tagged value, correct tag -> Ok
        let input: &[u8] = &hex! {"a0 03 0101ff"};
        let (rem, res) = T2::parse_ber(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res.is_some());

        // tagged value, correct tag and invalid content -> Fail
        let input: &[u8] = &hex! {"a0 03 0102ffff"};
        let _res = T2::parse_ber(input.into()).expect_err("parsing should have failed");
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod std_tests {
    use hex_literal::hex;

    use crate::{BerError, BerParser, Input, TaggedExplicit, ToBer};

    #[test]
    fn tober_tagged_explicit() {
        let mut v: Vec<u8> = Vec::new();

        type T<'a> = TaggedExplicit<bool, BerError<Input<'a>>, 0>;
        let t = T::explicit(true);
        v.clear();
        t.ber_encode(&mut v).expect("serialization failed");
        assert_eq!(&v, &hex! {"a0 03 0101ff"});

        // de-serialize to be sure
        let (rem, t2) = T::parse_ber((&v).into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(t, t2);
    }
}
