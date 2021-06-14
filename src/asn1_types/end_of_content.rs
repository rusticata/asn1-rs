use crate::{Any, Error, Result, Tag, Tagged};
use std::convert::TryFrom;

// No `FromDer` implementation, type is not valid in DER
#[derive(Debug)]
pub struct EndOfContent {}

impl EndOfContent {
    pub const fn new() -> Self {
        EndOfContent {}
    }
}

impl<'a> TryFrom<Any<'a>> for EndOfContent {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<EndOfContent> {
        any.tag().assert_eq(Self::TAG)?;
        if !any.header.length.is_null() {
            return Err(Error::InvalidLength);
        }
        Ok(EndOfContent {})
    }
}

impl<'a> Tagged for EndOfContent {
    const TAG: Tag = Tag::EndOfContent;
}
