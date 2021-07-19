use crate::{Any, Class, Error, FromBer, FromDer, Header, ParseResult, Tag};
use core::convert::TryFrom;

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

#[derive(Debug)]
pub struct TaggedOptionalExplicit<T, const N: u32> {
    pub(crate) inner: Option<T>,
}

impl<T, const N: u32> AsRef<Option<T>> for TaggedOptionalExplicit<T, N> {
    fn as_ref(&self) -> &Option<T> {
        &self.inner
    }
}

impl<'a, T, const N: u32> TryFrom<Any<'a>> for TaggedOptionalExplicit<T, N>
where
    T: FromBer<'a>,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self, Self::Error> {
        if any.tag().0 != N {
            Ok(TaggedOptionalExplicit { inner: None })
        } else {
            let (_, t) = T::from_ber(any.data)?;
            Ok(TaggedOptionalExplicit { inner: Some(t) })
        }
    }
}

#[derive(Debug)]
pub struct TaggedOptionalImplicit<T, const N: u32> {
    pub(crate) inner: Option<T>,
}

impl<T, const N: u32> AsRef<Option<T>> for TaggedOptionalImplicit<T, N> {
    fn as_ref(&self) -> &Option<T> {
        &self.inner
    }
}

impl<'a, T, const N: u32> TryFrom<Any<'a>> for TaggedOptionalImplicit<T, N>
where
    T: TryFrom<Any<'a>, Error = Error>,
{
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Self, Self::Error> {
        if any.tag().0 != N {
            Ok(TaggedOptionalImplicit { inner: None })
        } else {
            let any = Any {
                header: Header {
                    tag: Tag(N),
                    ..any.header.clone()
                },
                data: any.data,
            };
            match T::try_from(any) {
                Ok(t) => Ok(TaggedOptionalImplicit { inner: Some(t) }),
                Err(e) => Err(e),
            }
        }
    }
}
