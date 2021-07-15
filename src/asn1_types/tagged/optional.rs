use crate::{Any, Class, Error, FromBer, FromDer, Header, ParseResult, Tag};
use alloc::borrow::Cow;
use core::convert::TryFrom;

pub const TAG0: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(0));
pub const TAG1: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(1));
pub const TAG2: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(2));
pub const TAG3: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(3));
pub const TAG4: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(4));
pub const TAG5: TaggedOptional = TaggedOptional::new(Class::ContextSpecific, Tag(5));

#[derive(Debug)]
pub struct TaggedOptional {
    pub class: Class,
    pub tag: Tag,
}

impl TaggedOptional {
    pub const fn new(class: Class, tag: Tag) -> Self {
        TaggedOptional { class, tag }
    }

    // especially useful when parsing IMPLICIT
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
        let data = match any.data {
            Cow::Borrowed(b) => b,
            // any is just built from borrowed data, so we know it is unreachable
            Cow::Owned(_) => unreachable!(),
        };
        let header = any.header;
        let (_, res) = f(header, data)?;
        Ok((rem, Some(res)))
    }
}

// shortcut for ContextSpecific with known tag
impl From<Tag> for TaggedOptional {
    fn from(tag: Tag) -> Self {
        TaggedOptional::new(Class::ContextSpecific, tag)
    }
}

// shortcut for ContextSpecific with known tag
impl From<u32> for TaggedOptional {
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
            let data = match any.data {
                Cow::Borrowed(b) => b,
                Cow::Owned(_) => return Err(Error::LifetimeError),
            };
            let (_, t) = T::from_ber(data)?;
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
                data: any.into_cow(),
            };
            match T::try_from(any) {
                Ok(t) => Ok(TaggedOptionalImplicit { inner: Some(t) }),
                Err(e) => Err(e),
            }
        }
    }
}
