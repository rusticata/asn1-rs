use crate::{
    Any, CheckDerConstraints, Class, Error, FromBer, FromDer, Header, ParseResult, Result, Tag,
    Tagged,
};
use std::{borrow::Cow, convert::TryFrom, marker::PhantomData};

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
pub struct TaggedValue<'a, TagKind> {
    pub header: Header<'a>,
    pub data: Cow<'a, [u8]>,

    tag_kind: PhantomData<TagKind>,
}

impl<'a, TagKind> TaggedValue<'a, TagKind> {
    pub const fn new(header: Header<'a>, data: Cow<'a, [u8]>) -> Self {
        TaggedValue {
            header,
            data,
            tag_kind: PhantomData,
        }
    }

    pub fn from_expected_tag<IntoTag>(input: &'a [u8], tag: IntoTag) -> ParseResult<'a, Self>
    where
        IntoTag: Into<Tag>,
    {
        let (rem, Any { header, data }) = Any::from_ber(input)?;
        header.assert_tag(tag.into())?;
        Ok((rem, TaggedValue::new(header, data)))
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

impl<'a> TaggedValue<'a, Explicit> {
    pub fn parse_ber<T: FromBer<'a>>(&'a self) -> ParseResult<'a, T> {
        T::from_ber(&self.data)
    }

    pub fn parse_der<T: FromDer<'a>>(&'a self) -> ParseResult<'a, T> {
        T::from_der(&self.data)
    }
}

impl<'a> TaggedValue<'a, Implicit> {
    pub fn parse_ber<T>(&'a self) -> ParseResult<'a, T>
    where
        T: Tagged,
        T: TryFrom<Any<'a>, Error = Error>,
    {
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..self.header
            },
            data: Cow::Borrowed(&self.data),
        };
        match T::try_from(any) {
            Ok(t) => Ok((&[], t)),
            Err(e) => Err(nom::Err::Failure(e)),
        }
    }

    pub fn parse_der<T>(&'a self) -> ParseResult<'a, T>
    where
        T: Tagged,
        T: TryFrom<Any<'a>, Error = Error>,
        T: CheckDerConstraints,
    {
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..self.header
            },
            data: Cow::Borrowed(&self.data),
        };
        T::check_constraints(&any)?;
        match T::try_from(any) {
            Ok(t) => Ok((&[], t)),
            Err(e) => Err(nom::Err::Failure(e)),
        }
    }
}

impl<'a, TagKind> AsRef<[u8]> for TaggedValue<'a, TagKind> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<'a, TagKind> TryFrom<Any<'a>> for TaggedValue<'a, TagKind> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self> {
        let Any { header, data } = any;
        Ok(TaggedValue::new(header, data))
    }
}

impl<'a, TagKind> CheckDerConstraints for TaggedValue<'a, TagKind> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        Ok(())
    }
}
