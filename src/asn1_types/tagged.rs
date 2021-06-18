use crate::{
    Any, CheckDerConstraints, Class, Error, FromBer, FromDer, Header, ParseResult, Result, Tag,
    Tagged,
};
use nom::error::ParseError;
use nom::IResult;
use std::convert::TryFrom;
use std::{borrow::Cow, marker::PhantomData};

// tag class: universal, application, context-specific, private

// tag types: IMPLICIT, EXPLICIT

#[derive(Debug)]
pub enum Implicit {}

#[derive(Debug)]
pub enum Explicit {}

pub trait TagKind {}

impl TagKind for Implicit {}
impl TagKind for Explicit {}

#[derive(Debug, PartialEq)]
pub struct TaggedValue<'a, TagKind, T> {
    pub header: Header<'a>,
    pub inner: T,

    tag_kind: PhantomData<TagKind>,
}

impl<'a, TagKind, T> TaggedValue<'a, TagKind, T> {
    pub const fn new(header: Header<'a>, inner: T) -> Self {
        TaggedValue {
            header,
            inner,
            tag_kind: PhantomData,
        }
    }

    pub const fn assert_class(&self, class: Class) -> Result<()> {
        self.header.assert_class(class)
    }

    pub const fn assert_tag(&self, tag: Tag) -> Result<()> {
        self.header.assert_tag(tag)
    }

    pub const fn class(&self) -> Class {
        self.header.class
    }

    pub const fn tag(&self) -> Tag {
        self.header.tag
    }
}

impl<'a, TagKind, T> AsRef<T> for TaggedValue<'a, TagKind, T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T> TryFrom<Any<'a>> for TaggedValue<'a, Explicit, T>
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: Tagged,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        let header = any.header.clone();
        let data = match any.data {
            Cow::Borrowed(b) => b,
            Cow::Owned(_) => return Err(Error::LifetimeError),
        };
        let (_, inner_any) = Any::from_ber(data)?;
        let inner = T::try_from(inner_any)?;
        Ok(TaggedValue {
            header,
            inner,
            tag_kind: PhantomData,
        })
    }
}

impl<'a, T> TryFrom<Any<'a>> for TaggedValue<'a, Implicit, T>
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: Tagged,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        let Any { header, data } = any;
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..header.clone()
            },
            data,
        };
        match T::try_from(any) {
            Ok(t) => Ok(TaggedValue {
                header,
                inner: t,
                tag_kind: PhantomData,
            }),
            Err(e) => Err(e),
        }
    }
}

impl<'a, T> CheckDerConstraints for TaggedValue<'a, Explicit, T>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        let (_, inner_any) = Any::from_der(&any.data)?;
        T::check_constraints(&inner_any)?;
        Ok(())
    }
}

impl<'a, T> CheckDerConstraints for TaggedValue<'a, Implicit, T>
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
            data: Cow::Borrowed(&any.data),
        };
        T::check_constraints(&any)?;
        Ok(())
    }
}

impl<'a, TagKind, T> TaggedValue<'a, TagKind, T>
where
    Self: FromBer<'a>,
{
    pub fn parse_ber(class: Class, tag: Tag, bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, t) = TaggedValue::<TagKind, T>::from_ber(bytes)?;
        t.assert_class(class)?;
        t.assert_tag(tag)?;
        Ok((rem, t))
    }
}

impl<'a, TagKind, T> TaggedValue<'a, TagKind, T>
where
    Self: FromDer<'a>,
{
    pub fn parse_der(class: Class, tag: Tag, bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, t) = TaggedValue::<TagKind, T>::from_der(bytes)?;
        t.assert_class(class)?;
        t.assert_tag(tag)?;
        Ok((rem, t))
    }
}

// parser builder

#[derive(Clone, Copy, Debug)]
pub struct TagParser<TagKind> {
    class: Class,
    tag: Tag,
    tag_kind: PhantomData<TagKind>,
}

impl<TagKind> TagParser<TagKind> {
    pub const fn new() -> Self {
        TagParser {
            class: Class::Universal,
            tag: Tag(0),
            tag_kind: PhantomData,
        }
    }

    pub const fn with_class(self, class: Class) -> Self {
        Self { class, ..self }
    }

    pub const fn with_tag(self, tag: Tag) -> Self {
        Self { tag, ..self }
    }
}

impl TagParser<Explicit> {
    pub const fn explicit() -> Self {
        TagParser::new()
    }
}

impl TagParser<Implicit> {
    pub const fn implicit() -> Self {
        TagParser::new()
    }
}

impl<TagKind> TagParser<TagKind> {
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

// XXX how to add Class (universal/application/context-specific/private)?
pub fn parse_der_tagged_explicit<'a, IntoTag, T>(
    tag: IntoTag,
) -> impl FnMut(&'a [u8]) -> ParseResult<T>
where
    IntoTag: Into<Tag>,
    T: FromDer<'a>,
{
    let tag = tag.into();
    parse_der_tagged_explicit_g(tag, |content, _header| {
        let (rem, inner) = T::from_der(content)?;
        Ok((rem, inner))
    })
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
) -> impl FnMut(&'a [u8]) -> ParseResult<T>
where
    IntoTag: Into<Tag>,
    T: TryFrom<Any<'a>, Error = Error> + Tagged,
{
    let tag = tag.into();
    parse_der_tagged_implicit_g(tag, |content, _actual_tag, fake_header| {
        let any = Any::new(fake_header, content.into());

        let inner = T::try_from(any)?;
        Ok((&[], inner))
    })
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
