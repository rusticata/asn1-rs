use crate::*;
use core::convert::TryFrom;
use core::marker::PhantomData;

impl<T, E, const CLASS: u8, const TAG: u32> DynTagged for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: DynTagged,
{
    fn class(&self) -> Class {
        Class::try_from(CLASS).unwrap_or(Class::Private)
    }

    fn constructed(&self) -> bool {
        self.inner.constructed()
    }

    fn tag(&self) -> Tag {
        Tag(TAG)
    }

    fn accept_tag(tag: Tag) -> bool {
        tag.0 == TAG
    }
}

impl<'a, T, E, const CLASS: u8, const TAG: u32> TryFrom<Any<'a>>
    for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: TryFrom<Any<'a>, Error = E>,
    T: Tagged,
    E: From<Error>,
{
    type Error = E;

    fn try_from(any: Any<'a>) -> Result<Self, E> {
        TryFrom::try_from(&any)
    }
}

impl<'a, 'b, E, T, const CLASS: u8, const TAG: u32> TryFrom<&'b Any<'a>>
    for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: TryFrom<Any<'a>, Error = E>,
    T: Tagged,
    E: From<Error>,
{
    type Error = E;

    fn try_from(any: &'b Any<'a>) -> Result<Self, E> {
        any.tag().assert_eq(Tag(TAG))?;
        // XXX if input is empty, this function is not called

        if any.class() as u8 != CLASS {
            let class = Class::try_from(CLASS).ok();
            return Err(Error::unexpected_class(class, any.class()).into());
        }
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..any.header.clone()
            },
            data: any.data.clone(),
        };
        match T::try_from(any) {
            Ok(inner) => Ok(TaggedValue::implicit(inner)),
            Err(e) => Err(e),
        }
    }
}

impl<'i, T, E, const CLASS: u8, const TAG: u32> BerParser<'i>
    for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: BerParser<'i>,
    // E: ParseError<Input<'a>> + From<BerError<Input<'a>>>,
{
    type Error = T::Error;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // assert class
        if header.class as u8 != CLASS {
            // Safety: CLASS < 4
            let class = Class::try_from(CLASS).unwrap_or(Class::Private);
            return Err(Err::Error(
                BerError::unexpected_class(input, Some(class), header.class).into(),
            ));
        }

        // pass the same header to parse inner content
        // note: we *know* that header.tag is most probably different from t::tag,
        // so the tag is not checked here

        let (rem, t) = T::from_ber_content(header, input)?;
        let tagged = TaggedValue::implicit(t);
        Ok((rem, tagged))
    }
}

impl<'i, T, E, const CLASS: u8, const TAG: u32> DerParser<'i>
    for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: DerParser<'i>,
    // E: ParseError<Input<'a>> + From<BerError<Input<'a>>>,
{
    type Error = T::Error;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // assert class
        if header.class as u8 != CLASS {
            // Safety: CLASS < 4
            let class = Class::try_from(CLASS).unwrap_or(Class::Private);
            return Err(Err::Error(
                BerError::unexpected_class(input, Some(class), header.class).into(),
            ));
        }

        // pass the same header to parse inner content
        // note: we *know* that header.tag is most probably different from t::tag,
        // so the tag is not checked here

        let (rem, t) = T::from_der_content(header, input)?;
        let tagged = TaggedValue::implicit(t);
        Ok((rem, tagged))
    }
}

impl<'a, T, E, const CLASS: u8, const TAG: u32> FromDer<'a, E>
    for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: TryFrom<Any<'a>, Error = E>,
    T: Tagged,
    E: From<Error>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        let (rem, any) = Any::from_der(bytes).map_err(Err::convert)?;
        any.tag()
            .assert_eq(Tag(TAG))
            .map_err(|e| Err::Error(e.into()))?;
        if any.class() as u8 != CLASS {
            let class = Class::try_from(CLASS).ok();
            return Err(Err::Error(
                Error::unexpected_class(class, any.class()).into(),
            ));
        }
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..any.header.clone()
            },
            data: any.data,
        };
        match T::try_from(any) {
            Ok(inner) => Ok((rem, TaggedValue::implicit(inner))),
            Err(e) => Err(Err::Error(e)),
        }
    }
}

impl<T, E, const CLASS: u8, const TAG: u32> CheckDerConstraints
    for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: CheckDerConstraints,
    T: Tagged,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let header = any.header.clone().with_tag(T::TAG);
        let inner = Any::new(header, any.data.clone());
        T::check_constraints(&inner)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T, E, const CLASS: u8, const TAG: u32> ToDer for TaggedValue<T, E, Implicit, CLASS, TAG>
where
    T: ToDer + DynTagged,
{
    fn to_der_len(&self) -> Result<usize> {
        self.inner.to_der_len()
    }

    fn write_der(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let class =
            Class::try_from(CLASS).map_err(|_| SerializeError::InvalidClass { class: CLASS })?;
        let mut v = Vec::new();
        let inner_len = self.inner.write_der_content(&mut v)?;
        // XXX X.690 section 8.14.3: if implicing tagging was used [...]:
        // XXX a) the encoding shall be constructed if the base encoding is constructed, and shall be primitive otherwise
        let constructed = matches!(self.inner.tag(), Tag::Sequence | Tag::Set);
        let header = Header::new(class, constructed, self.tag(), Length::Definite(inner_len));
        let sz = header.write_der_header(writer)?;
        let sz = sz + writer.write(&v)?;
        Ok(sz)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut sink = std::io::sink();
        let class =
            Class::try_from(CLASS).map_err(|_| SerializeError::InvalidClass { class: CLASS })?;
        let inner_len = self.inner.write_der_content(&mut sink)?;
        // XXX X.690 section 8.14.3: if implicing tagging was used [...]:
        // XXX a) the encoding shall be constructed if the base encoding is constructed, and shall be primitive otherwise
        let constructed = matches!(self.inner.tag(), Tag::Sequence | Tag::Set);
        let header = Header::new(class, constructed, self.tag(), Length::Definite(inner_len));
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        self.inner.write_der_content(writer)
    }
}

#[cfg(feature = "std")]
const _: () = {
    impl<T, E, const CLASS: u8, const TAG: u32> ToBer for TaggedValue<T, E, Implicit, CLASS, TAG>
    where
        T: ToBer,
        T: DynTagged,
    {
        type Encoder = BerGenericEncoder<TaggedValue<T, E, Implicit, CLASS, TAG>>;

        fn content_len(&self) -> Length {
            self.inner.content_len()
        }

        fn write_content<W: std::io::Write>(&self, target: &mut W) -> SerializeResult<usize> {
            self.inner.write_content(target)
        }
    }
};

/// A helper object to parse `[ n ] IMPLICIT T`
///
/// A helper object implementing [`FromBer`] and [`FromDer`], to parse tagged
/// optional values.
///
/// This helper expects context-specific tags.
/// See [`TaggedValue`] or [`TaggedParser`] for more generic implementations if needed.
///
/// # Examples
///
/// To parse a `[0] IMPLICIT INTEGER OPTIONAL` object:
///
/// ```rust
/// use asn1_rs::{Error, FromBer, Integer, TaggedImplicit, TaggedValue};
///
/// let bytes = &[0x80, 0x1, 0x2];
///
/// let (_, tagged) = TaggedImplicit::<Integer, Error, 0>::from_ber(bytes).unwrap();
/// assert_eq!(tagged, TaggedValue::implicit(Integer::from(2)));
/// ```
pub type TaggedImplicit<T, E, const TAG: u32> = TaggedValue<T, E, Implicit, CONTEXT_SPECIFIC, TAG>;

impl<'a, T, E> FromBer<'a, E> for TaggedParser<'a, Implicit, T, E>
where
    T: TryFrom<Any<'a>, Error = E>,
    T: Tagged,
    E: From<Error>,
{
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        let (rem, any) = Any::from_ber(bytes).map_err(Err::convert)?;
        let Any { header, data } = any;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..header.clone()
            },
            data,
        };
        match T::try_from(any) {
            Ok(t) => {
                let tagged_value = TaggedParser {
                    header,
                    inner: t,
                    tag_kind: PhantomData,
                    _e: PhantomData,
                };
                Ok((rem, tagged_value))
            }
            Err(e) => Err(Err::Error(e)),
        }
    }
}

// implementations for TaggedParser

impl<T, E> TaggedParser<'_, Implicit, T, E> {
    pub const fn new_implicit(class: Class, constructed: bool, tag: u32, inner: T) -> Self {
        Self {
            header: Header::new(class, constructed, Tag(tag), Length::Definite(0)),
            inner,
            tag_kind: PhantomData,
            _e: PhantomData,
        }
    }
}

impl<'a, T, E> FromDer<'a, E> for TaggedParser<'a, Implicit, T, E>
where
    T: TryFrom<Any<'a>, Error = E>,
    T: CheckDerConstraints,
    T: Tagged,
    E: From<Error>,
{
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self, E> {
        let (rem, any) = Any::from_der(bytes).map_err(Err::convert)?;
        let Any { header, data } = any;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..header.clone()
            },
            data,
        };
        T::check_constraints(&any).map_err(|e| Err::Error(e.into()))?;
        match T::try_from(any) {
            Ok(t) => {
                let tagged_value = TaggedParser {
                    header,
                    inner: t,
                    tag_kind: PhantomData,
                    _e: PhantomData,
                };
                Ok((rem, tagged_value))
            }
            Err(e) => Err(Err::Error(e)),
        }
    }
}

impl<T> CheckDerConstraints for TaggedParser<'_, Implicit, T>
where
    T: CheckDerConstraints,
    T: Tagged,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..any.header.clone()
            },
            data: any.data.clone(),
        };
        T::check_constraints(&any)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<T> ToDer for TaggedParser<'_, Implicit, T>
where
    T: ToDer,
{
    fn to_der_len(&self) -> Result<usize> {
        self.inner.to_der_len()
    }

    fn write_der(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut v = Vec::new();
        let inner_len = self.inner.write_der_content(&mut v)?;
        // XXX X.690 section 8.14.3: if implicing tagging was used [...]:
        // XXX a) the encoding shall be constructed if the base encoding is constructed, and shall be primitive otherwise
        let header = Header::new(self.class(), false, self.tag(), Length::Definite(inner_len));
        let sz = header.write_der_header(writer)?;
        let sz = sz + writer.write(&v)?;
        Ok(sz)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let mut sink = std::io::sink();
        let inner_len = self.inner.write_der_content(&mut sink)?;
        // XXX X.690 section 8.14.3: if implicing tagging was used [...]:
        // XXX a) the encoding shall be constructed if the base encoding is constructed, and shall be primitive otherwise
        let header = Header::new(self.class(), false, self.tag(), Length::Definite(inner_len));
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        self.inner.write_der_content(writer)
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerError, BerParser, Input, TaggedImplicit};

    #[test]
    fn check_tagged_implicit() {
        type T<'a> = TaggedImplicit<bool, BerError<Input<'a>>, 0>;

        // untagged value -> should fail
        let input: &[u8] = &hex! {"0101ff"};
        let _res = T::parse_ber(input.into()).expect_err("parsing should have failed");

        // tagged value, correct tag -> Ok
        let input: &[u8] = &hex! {"8001ff"};
        let _res = T::parse_ber(input.into()).expect("parsing failed");

        // tagged value, incorrect tag -> Fail
        let input: &[u8] = &hex! {"8101ff"};
        let _res = T::parse_ber(input.into()).expect_err("parsing should have failed");

        // tagged value, correct tag but incorrect class -> Fail
        let input: &[u8] = &hex! {"4001ff"};
        let _res = T::parse_ber(input.into()).expect_err("parsing should have failed");
    }

    #[test]
    fn check_opttagged_implicit() {
        // **** using parse_ber_optional ****
        type T1<'a> = TaggedImplicit<bool, BerError<Input<'a>>, 0>;

        // empty -> OK, should return None
        let input: &[u8] = &hex! {""};
        let res = T1::parse_ber_optional(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // incorrect tag -> OK, should return None
        let input: &[u8] = &hex! {"8101ff"};
        let res = T1::parse_ber_optional(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // tagged value, correct tag -> Ok
        let input: &[u8] = &hex! {"8001ff"};
        let (rem, res) = T1::parse_ber_optional(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res.is_some());

        // tagged value, correct tag and invalid content -> Fail
        let input: &[u8] = &hex! {"8002ffff"};
        let _res = T1::parse_ber_optional(input.into()).expect_err("parsing should have failed");

        // **** using Option<T> ****
        type T2<'a> = Option<TaggedImplicit<bool, BerError<Input<'a>>, 0>>;

        // empty -> OK, should return None
        let input: &[u8] = &hex! {""};
        let res = T2::parse_ber(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // incorrect tag -> OK, should return None
        let input: &[u8] = &hex! {"8101ff"};
        let res = T2::parse_ber(input.into()).expect("parsing failed");
        assert_eq!(res.1, None);

        // tagged value, correct tag -> Ok
        let input: &[u8] = &hex! {"8001ff"};
        let (rem, res) = T2::parse_ber(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res.is_some());

        // tagged value, correct tag and invalid content -> Fail
        let input: &[u8] = &hex! {"8002ffff"};
        let _res = T2::parse_ber(input.into()).expect_err("parsing should have failed");
    }
}

#[cfg(feature = "std")]
#[cfg(test)]
mod std_tests {
    use hex_literal::hex;

    use crate::{BerError, BerParser, Input, TaggedImplicit, ToBer};

    #[test]
    fn tober_tagged_implicit() {
        let mut v: Vec<u8> = Vec::new();

        type T<'a> = TaggedImplicit<bool, BerError<Input<'a>>, 0>;
        let t = T::implicit(true);
        v.clear();
        t.encode(&mut v).expect("serialization failed");
        assert_eq!(&v, &hex! {"8001ff"});

        // de-serialize to be sure
        let (rem, t2) = T::parse_ber((&v).into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(t, t2);
    }
}
