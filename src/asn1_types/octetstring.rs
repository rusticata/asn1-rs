use crate::CheckDerConstraints;
use crate::{Any, Error, Result, Tag, Tagged};
use std::borrow::Cow;
use std::convert::TryFrom;

#[derive(Debug)]
pub struct OctetString<'a> {
    data: Cow<'a, [u8]>,
}

impl<'a> OctetString<'a> {
    pub const fn new(s: &'a [u8]) -> Self {
        OctetString {
            data: Cow::Borrowed(s),
        }
    }
}

impl<'a> AsRef<[u8]> for OctetString<'a> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<'a> TryFrom<Any<'a>> for OctetString<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<OctetString<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        Ok(OctetString {
            data: any.into_cow(),
        })
    }
}

impl<'a> CheckDerConstraints for OctetString<'a> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for OctetString<'a> {
    const TAG: Tag = Tag::OctetString;
}
