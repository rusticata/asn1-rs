use crate::{ber::*, FromBer, FromDer, Header, ParseResult, Tag, ToStatic};
use std::borrow::Cow;

#[derive(Debug)]
pub struct Any<'a> {
    pub header: Header<'a>,
    pub data: Cow<'a, [u8]>,
}

impl<'a> Any<'a> {
    pub const fn tag(&self) -> Tag {
        self.header.tag
    }

    /// Get the bytes representation of the *content*
    pub fn as_cow(&'a self) -> &Cow<'a, [u8]> {
        &self.data
    }

    /// Get the bytes representation of the *content*
    pub fn into_cow(self) -> Cow<'a, [u8]> {
        self.data
    }

    /// Get the bytes representation of the *content*
    pub fn as_bytes(&'a self) -> &'a [u8] {
        &self.data
    }
}

impl<'a> FromBer<'a> for Any<'a> {
    fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
        let (i, header) = Header::from_ber(bytes)?;
        let (i, data) = ber_get_object_content(i, &header, MAX_RECURSION)?;
        let data = Cow::Borrowed(data);
        Ok((i, Any { header, data }))
    }
}

impl<'a> FromDer<'a> for Any<'a> {
    fn from_der(bytes: &'a [u8]) -> ParseResult<Self> {
        let (i, header) = Header::from_der(bytes)?;
        // X.690 section 10.1: The definite form of length encoding shall be used
        header.length.assert_definite()?;
        let (i, data) = ber_get_object_content(i, &header, MAX_RECURSION)?;
        let data = Cow::Borrowed(data);
        Ok((i, Any { header, data }))
    }
}

impl<'a> ToStatic for Any<'a> {
    type Owned = Any<'static>;

    fn to_static(&self) -> Self::Owned {
        Any {
            header: self.header.to_static(),
            data: Cow::Owned(self.data.to_vec()),
        }
    }
}
