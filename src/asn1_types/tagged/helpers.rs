use super::{Explicit, Implicit, TaggedValue};
use crate::{Any, Class, Error, FromBer, FromDer, Header, ParseResult, Tag, Tagged};
use alloc::borrow::Cow;
use core::marker::PhantomData;
use nom::error::ParseError;
use nom::IResult;

/// A builder for parsing tagged values (`IMPLICIT` or `EXPLICIT`)
///
/// # Examples
///
/// ```
/// use asn1_rs::{Class, Tag, TagParser};
///
/// let parser = TagParser::explicit()
///     .with_class(Class::ContextSpecific)
///     .with_tag(Tag(0))
///     .der_parser::<u32>();
///
/// let input = &[0xa0, 0x03, 0x02, 0x01, 0x02];
/// let (rem, tagged) = parser(input).expect("parsing failed");
///
/// assert!(rem.is_empty());
/// assert_eq!(tagged.tag(), Tag(0));
/// assert_eq!(tagged.as_ref(), &2);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct TagParser<TagKind> {
    class: Class,
    tag: Tag,
    tag_kind: PhantomData<TagKind>,
}

impl<TagKind> TagParser<TagKind> {
    /// Create a default `TagParser` builder
    ///
    /// `TagKind` must be specified as either [`Explicit`] or [`Implicit`]
    ///
    /// ```
    /// use asn1_rs::{Explicit, TagParser};
    ///
    /// let builder = TagParser::<Explicit>::new();
    /// ```
    pub const fn new() -> Self {
        TagParser {
            class: Class::Universal,
            tag: Tag(0),
            tag_kind: PhantomData,
        }
    }

    /// Set the expected `Class` for the builder
    pub const fn with_class(self, class: Class) -> Self {
        Self { class, ..self }
    }

    /// Set the expected `Tag` for the builder
    pub const fn with_tag(self, tag: Tag) -> Self {
        Self { tag, ..self }
    }
}

impl TagParser<Explicit> {
    /// Create a `TagParser` builder for `EXPLICIT` tagged values
    pub const fn explicit() -> Self {
        TagParser::new()
    }
}

impl TagParser<Implicit> {
    /// Create a `TagParser` builder for `IMPLICIT` tagged values
    pub const fn implicit() -> Self {
        TagParser::new()
    }
}

impl<TagKind> TagParser<TagKind> {
    /// Create the BER parser from the builder parameters
    ///
    /// This method will consume the builder and return a parser (to be used as a function).
    pub fn ber_parser<'a, T>(
        self,
    ) -> impl Fn(&'a [u8]) -> ParseResult<'a, TaggedValue<'a, TagKind, T>>
    where
        TaggedValue<'a, TagKind, T>: FromBer<'a>,
    {
        move |bytes: &[u8]| TaggedValue::<TagKind, T>::parse_ber(self.class, self.tag, bytes)
    }
}

impl<TagKind> TagParser<TagKind> {
    /// Create the DER parser from the builder parameters
    ///
    /// This method will consume the builder and return a parser (to be used as a function).
    pub fn der_parser<'a, T>(
        self,
    ) -> impl Fn(&'a [u8]) -> ParseResult<'a, TaggedValue<'a, TagKind, T>>
    where
        TaggedValue<'a, TagKind, T>: FromDer<'a>,
    {
        move |bytes: &[u8]| TaggedValue::<TagKind, T>::parse_der(self.class, self.tag, bytes)
    }
}

// helper functions for parsing tagged objects

pub fn parse_der_tagged_explicit<'a, IntoTag, T>(
    tag: IntoTag,
) -> impl FnMut(&'a [u8]) -> ParseResult<TaggedValue<'a, Explicit, T>>
where
    IntoTag: Into<Tag>,
    TaggedValue<'a, Explicit, T>: FromDer<'a>,
{
    let tag = tag.into();
    move |i| {
        let (rem, tagged) = TaggedValue::from_der(i)?;
        tagged.assert_tag(tag)?;
        Ok((rem, tagged))
    }
}

pub fn parse_der_tagged_explicit_g<'a, IntoTag, T, F, E>(
    tag: IntoTag,
    f: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    F: Fn(&'a [u8], Header<'a>) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]> + From<Error>,
    IntoTag: Into<Tag>,
{
    let tag = tag.into();
    parse_der_container(tag, move |any: Any<'a>| {
        any.header
            .assert_tag(tag)
            .map_err(|e| nom::Err::convert(e.into()))?;
        let data: &'a [u8] = match any.data {
            Cow::Borrowed(b) => b,
            Cow::Owned(_) => unreachable!(),
        };
        f(data, any.header)
    })
}

pub fn parse_der_tagged_implicit<'a, IntoTag, T>(
    tag: IntoTag,
) -> impl FnMut(&'a [u8]) -> ParseResult<TaggedValue<'a, Implicit, T>>
where
    IntoTag: Into<Tag>,
    // T: TryFrom<Any<'a>, Error = Error> + Tagged,
    TaggedValue<'a, Implicit, T>: FromDer<'a>,
{
    let tag = tag.into();
    move |i| {
        let (rem, tagged) = TaggedValue::from_der(i)?;
        tagged.assert_tag(tag)?;
        Ok((rem, tagged))
    }
}

pub fn parse_der_tagged_implicit_g<'a, IntoTag, T, F, E>(
    tag: IntoTag,
    f: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    F: Fn(&'a [u8], Tag, Header<'a>) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]> + From<Error>,
    IntoTag: Into<Tag>,
    T: Tagged,
{
    let tag = tag.into();
    parse_der_container(tag, move |any: Any<'a>| {
        // verify tag of external header
        any.header
            .assert_tag(tag)
            .map_err(|e| nom::Err::convert(e.into()))?;
        // build a fake header with the expected tag
        let Any { header, data } = any;
        let header = Header {
            tag: T::TAG,
            ..header.clone()
        };
        let data: &'a [u8] = match data {
            Cow::Borrowed(b) => b,
            Cow::Owned(_) => unreachable!(),
        };
        f(data, tag, header)
    })
}

fn parse_der_container<'a, T, F, E>(
    tag: Tag,
    f: F,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], T, E>
where
    F: Fn(Any<'a>) -> IResult<&'a [u8], T, E>,
    E: ParseError<&'a [u8]> + From<Error>,
{
    move |i: &[u8]| {
        let (rem, any) = Any::from_der(i).map_err(nom::Err::convert)?;
        any.header
            .assert_tag(tag)
            .map_err(|e| nom::Err::convert(e.into()))?;
        let (_, output) = f(any)?;
        Ok((rem, output))
    }
}
