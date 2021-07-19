use crate::{
    Any, CheckDerConstraints, Class, Error, Explicit, FromBer, FromDer, Header, Implicit,
    ParseResult, Tag, Tagged,
};
use core::convert::TryFrom;
use std::marker::PhantomData;

pub const TAG_OPT0: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(0));
pub const TAG_OPT1: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(1));
pub const TAG_OPT2: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(2));
pub const TAG_OPT3: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(3));
pub const TAG_OPT4: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(4));
pub const TAG_OPT5: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(5));

/// Helper object to parse TAGGED OPTIONAL types (explicit or implicit)
///
/// This object can be used similarly to a builder pattern, to specify the expected class and
/// tag of the object to parse, and the content parsing function.
///
/// The content parsing function takes two arguments: the outer header, and the data.
///
/// It can be used for both EXPLICIT or IMPLICIT tagged objects by using parsing functions that
/// expect a header (or not) in the contents.
///
/// The [`TaggedOptional::from`] method is a shortcut to build an object with `ContextSpecific`
/// class and the given tag. The [`TaggedOptional::new`] method is more generic.
///
/// # Examples
///
/// To parse a `[APPLICATION 0] EXPLICIT INTEGER OPTIONAL` object:
///
/// ```rust
/// use asn1_rs::{Class, FromDer, Integer, Tag, TaggedOptional};
///
/// let bytes = &[0x60, 0x03, 0x2, 0x1, 0x2];
///
/// let (_, tagged) = TaggedOptional::new(Class::Application, Tag(0))
///                     .parse_der(bytes, |_, data| Integer::from_der(data))
///                     .unwrap();
///
/// assert_eq!(tagged, Some(Integer::from(2)));
/// ```
///
/// To parse a `[0] IMPLICIT INTEGER OPTIONAL` object:
///
/// ```rust
/// use asn1_rs::{Integer, TaggedOptional};
///
/// let bytes = &[0xa0, 0x1, 0x2];
///
/// let (_, tagged) = TaggedOptional::from(0)
///                     .parse_der(bytes, |_, data| Ok((&[], Integer::new(data))))
///                     .unwrap();
///
/// assert_eq!(tagged, Some(Integer::from(2)));
/// ```
#[derive(Debug)]
pub struct TaggedOptional {
    /// The expected class for the object to parse
    pub class: Class,
    /// The expected tag for the object to parse
    pub tag: Tag,
}

impl TaggedOptional {
    /// Build a new `TaggedOptional` object.
    ///
    /// If using `Class::ContextSpecific`, using [`TaggedOptional::from`] with either a `Tag` or `u32` is
    /// a shorter way to build this object.
    pub const fn new(class: Class, tag: Tag) -> Self {
        TaggedOptional { class, tag }
    }

    /// Parse input as BER, and apply the provided function to parse object.
    ///
    /// Returns the remaining bytes, and `Some(T)` if expected tag was found, else `None`.
    ///
    ///  This function returns an error if tag was found but has a different class, or if parsing fails.
    ///
    /// # Examples
    ///
    /// To parse a `[0] EXPLICIT INTEGER OPTIONAL` object:
    ///
    /// ```rust
    /// use asn1_rs::{FromBer, Integer, TaggedOptional};
    ///
    /// let bytes = &[0xa0, 0x03, 0x2, 0x1, 0x2];
    ///
    /// let (_, tagged) = TaggedOptional::from(0)
    ///                     .parse_ber(bytes, |_, data| Integer::from_ber(data))
    ///                     .unwrap();
    ///
    /// assert_eq!(tagged, Some(Integer::from(2)));
    /// ```
    pub fn parse_ber<'a, T, F>(&self, bytes: &'a [u8], f: F) -> ParseResult<'a, Option<T>>
    where
        F: Fn(Header, &'a [u8]) -> ParseResult<'a, T>,
    {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        let (rem, any) = Any::from_ber(bytes)?;
        if any.tag() != self.tag {
            return Ok((bytes, None));
        }
        if any.class() != self.class {
            return Err(Error::UnexpectedClass(any.class()).into());
        }
        let Any { header, data } = any;
        let (_, res) = f(header, data)?;
        Ok((rem, Some(res)))
    }

    /// Parse input as DER, and apply the provided function to parse object.
    ///
    /// Returns the remaining bytes, and `Some(T)` if expected tag was found, else `None`.
    ///
    ///  This function returns an error if tag was found but has a different class, or if parsing fails.
    ///
    /// # Examples
    ///
    /// To parse a `[0] EXPLICIT INTEGER OPTIONAL` object:
    ///
    /// ```rust
    /// use asn1_rs::{FromDer, Integer, TaggedOptional};
    ///
    /// let bytes = &[0xa0, 0x03, 0x2, 0x1, 0x2];
    ///
    /// let (_, tagged) = TaggedOptional::from(0)
    ///                     .parse_der(bytes, |_, data| Integer::from_der(data))
    ///                     .unwrap();
    ///
    /// assert_eq!(tagged, Some(Integer::from(2)));
    /// ```
    pub fn parse_der<'a, T, F>(&self, bytes: &'a [u8], f: F) -> ParseResult<'a, Option<T>>
    where
        F: Fn(Header, &'a [u8]) -> ParseResult<'a, T>,
    {
        if bytes.is_empty() {
            return Ok((bytes, None));
        }
        let (rem, any) = Any::from_der(bytes)?;
        if any.tag() != self.tag {
            return Ok((bytes, None));
        }
        if any.class() != self.class {
            return Err(Error::UnexpectedClass(any.class()).into());
        }
        let Any { header, data } = any;
        let (_, res) = f(header, data)?;
        Ok((rem, Some(res)))
    }
}

impl From<Tag> for TaggedOptional {
    /// Build a `TaggedOptional` object with class `ContextSpecific` and given tag
    #[inline]
    fn from(tag: Tag) -> Self {
        TaggedOptional::new(Class::ContextSpecific, tag)
    }
}

impl From<u32> for TaggedOptional {
    /// Build a `TaggedOptional` object with class `ContextSpecific` and given tag
    #[inline]
    fn from(tag: u32) -> Self {
        TaggedOptional::new(Class::ContextSpecific, Tag(tag))
    }
}

//
// XXX 2nd try, with a const generic to contain the tag number (class ?)
// this allows directly returning an object with TryFrom<Any>

/// Helper object for creating `FromBer`/`FromDer` types for TAGGED OPTIONAL types
///
/// When parsing `ContextSpecific` (most common class), see [`TaggedExplicit`] and
/// [`TaggedImplicit`] alias types.
///
/// # Examples
///
/// To parse a `[APPLICATION 0] EXPLICIT INTEGER` object:
///
/// ```rust
/// use asn1_rs::{Explicit, FromBer, Integer, TaggedParser};
///
/// let bytes = &[0x60, 0x03, 0x2, 0x1, 0x2];
///
/// // If tagged object is present (and has expected tag), parsing succeeds:
/// let (_, tagged) = TaggedParser::<Integer, Explicit, 0b01, 0>::from_ber(bytes).unwrap();
/// assert_eq!(tagged, TaggedParser::explicit(Integer::from(2)));
/// ```
#[derive(Debug, PartialEq)]
pub struct TaggedParser<T, TagKind, const CLASS: u8, const TAG: u32> {
    pub(crate) inner: T,

    tag_kind: PhantomData<TagKind>,
}

impl<T, TagKind, const CLASS: u8, const TAG: u32> TaggedParser<T, TagKind, CLASS, TAG> {
    /// Consumes the `TaggedParser`, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T, const CLASS: u8, const TAG: u32> TaggedParser<T, Explicit, CLASS, TAG> {
    /// Constructs a new `EXPLICIT TaggedParser` with the provided value
    #[inline]
    pub const fn explicit(inner: T) -> Self {
        TaggedParser {
            inner,
            tag_kind: PhantomData,
        }
    }
}

impl<T, const CLASS: u8, const TAG: u32> TaggedParser<T, Implicit, CLASS, TAG> {
    /// Constructs a new `IMPLICIT TaggedParser` with the provided value
    #[inline]
    pub const fn implicit(inner: T) -> Self {
        TaggedParser {
            inner,
            tag_kind: PhantomData,
        }
    }
}

impl<T, TagKind, const CLASS: u8, const TAG: u32> AsRef<T>
    for TaggedParser<T, TagKind, CLASS, TAG>
{
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T, const CLASS: u8, const TAG: u32> TryFrom<Any<'a>>
    for TaggedParser<T, Explicit, CLASS, TAG>
where
    T: FromBer<'a>,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self, Self::Error> {
        any.tag().assert_eq(Tag(TAG))?;
        any.header.assert_constructed()?;
        if any.class() as u8 != CLASS {
            return Err(Error::UnexpectedClass(any.class()));
        }
        let (_, inner) = T::from_ber(any.data)?;
        Ok(TaggedParser::explicit(inner))
    }
}

impl<'a, T, const CLASS: u8, const TAG: u32> CheckDerConstraints
    for TaggedParser<T, Explicit, CLASS, TAG>
where
    T: CheckDerConstraints,
{
    fn check_constraints(any: &Any) -> crate::Result<()> {
        any.header.length.assert_definite()?;
        let (_, inner) = Any::from_ber(&any.data)?;
        T::check_constraints(&inner)?;
        Ok(())
    }
}

impl<'a, T, const CLASS: u8, const TAG: u32> TryFrom<Any<'a>>
    for TaggedParser<T, Implicit, CLASS, TAG>
where
    T: TryFrom<Any<'a>, Error = Error>,
    T: Tagged,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self, Self::Error> {
        any.tag().assert_eq(Tag(TAG))?;
        // XXX if input is empty, this function is not called

        if any.class() as u8 != CLASS {
            return Err(Error::UnexpectedClass(any.class()));
        }
        let any = Any {
            header: Header {
                tag: T::TAG,
                ..any.header.clone()
            },
            data: any.data,
        };
        match T::try_from(any) {
            Ok(inner) => Ok(TaggedParser::implicit(inner)),
            Err(e) => Err(e),
        }
    }
}

impl<'a, T, const CLASS: u8, const TAG: u32> CheckDerConstraints
    for TaggedParser<T, Implicit, CLASS, TAG>
where
    T: CheckDerConstraints,
    T: Tagged,
{
    fn check_constraints(any: &Any) -> crate::Result<()> {
        any.header.length.assert_definite()?;
        let header = any.header.clone().with_tag(T::TAG);
        let inner = Any::new(header, any.data);
        T::check_constraints(&inner)?;
        Ok(())
    }
}

const CONTEXT_SPECIFIC: u8 = Class::ContextSpecific as u8;

/// A helper object to parse `[ 0 ] EXPLICIT T`
///
/// A helper object implementing [`FromBer`] and [`FromDer`], to parse tagged
/// optional values.
///
/// This helper expects context-specific tags.
/// Use [`TaggedParser`] for a more generic implementation.
///
/// # Examples
///
/// To parse a `[0] EXPLICIT INTEGER` object:
///
/// ```rust
/// use asn1_rs::{FromBer, Integer, TaggedExplicit, TaggedParser};
///
/// let bytes = &[0xa0, 0x03, 0x2, 0x1, 0x2];
///
/// // If tagged object is present (and has expected tag), parsing succeeds:
/// let (_, tagged) = TaggedExplicit::<Integer, 0>::from_ber(bytes).unwrap();
/// assert_eq!(tagged, TaggedParser::explicit(Integer::from(2)));
/// ```
pub type TaggedExplicit<T, const TAG: u32> = TaggedParser<T, Explicit, CONTEXT_SPECIFIC, TAG>;

/// A helper object to parse `[ n ] EXPLICIT T OPTIONAL`
///
/// A helper object implementing [`FromBer`] and [`FromDer`], to parse tagged
/// optional values.
///
/// This helper expects context-specific tags.
/// Use `Option<` [`TaggedParser`] `>` for a more generic implementation.
///
/// # Examples
///
/// To parse a `[0] EXPLICIT INTEGER OPTIONAL` object:
///
/// ```rust
/// use asn1_rs::{FromBer, Integer, OptTaggedExplicit, TaggedParser};
///
/// let bytes = &[0xa0, 0x03, 0x2, 0x1, 0x2];
///
/// // If tagged object is present (and has expected tag), parsing succeeds:
/// let (_, tagged) = OptTaggedExplicit::<Integer, 0>::from_ber(bytes).unwrap();
/// assert_eq!(tagged, Some(TaggedParser::explicit(Integer::from(2))));
///
/// // If tagged object is not present or has different tag, parsing
/// // also succeeds (returning None):
/// let (_, tagged) = OptTaggedExplicit::<Integer, 0>::from_ber(&[]).unwrap();
/// assert_eq!(tagged, None);
/// ```
pub type OptTaggedExplicit<T, const TAG: u32> = Option<TaggedExplicit<T, TAG>>;

/// A helper object to parse `[ n ] IMPLICIT T`
///
/// A helper object implementing [`FromBer`] and [`FromDer`], to parse tagged
/// optional values.
///
/// This helper expects context-specific tags.
/// Use `Option<` [`TaggedParser`] `>` for a more generic implementation.
///
/// # Examples
///
/// To parse a `[0] IMPLICIT INTEGER OPTIONAL` object:
///
/// ```rust
/// use asn1_rs::{FromBer, Integer, TaggedImplicit, TaggedParser};
///
/// let bytes = &[0xa0, 0x1, 0x2];
///
/// let (_, tagged) = TaggedImplicit::<Integer, 0>::from_ber(bytes).unwrap();
/// assert_eq!(tagged, TaggedParser::implicit(Integer::from(2)));
/// ```
pub type TaggedImplicit<T, const TAG: u32> = TaggedParser<T, Implicit, CONTEXT_SPECIFIC, TAG>;

/// A helper object to parse `[ n ] IMPLICIT T OPTIONAL`
///
/// A helper object implementing [`FromBer`] and [`FromDer`], to parse tagged
/// optional values.
///
/// This helper expects context-specific tags.
/// Use `Option<` [`TaggedParser`] `>` for a more generic implementation.
///
/// # Examples
///
/// To parse a `[0] IMPLICIT INTEGER OPTIONAL` object:
///
/// ```rust
/// use asn1_rs::{FromBer, Integer, OptTaggedImplicit, TaggedParser};
///
/// let bytes = &[0xa0, 0x1, 0x2];
///
/// let (_, tagged) = OptTaggedImplicit::<Integer, 0>::from_ber(bytes).unwrap();
/// assert_eq!(tagged, Some(TaggedParser::implicit(Integer::from(2))));
///
/// // If tagged object is not present or has different tag, parsing
/// // also succeeds (returning None):
/// let (_, tagged) = OptTaggedImplicit::<Integer, 0>::from_ber(&[]).unwrap();
/// assert_eq!(tagged, None);
/// ```
pub type OptTaggedImplicit<T, const TAG: u32> = Option<TaggedImplicit<T, TAG>>;
