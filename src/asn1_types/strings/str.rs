use crate::{
    Any, CheckDerConstraints, Class, Error, Header, Length, Result, SerializeResult, Tag, Tagged,
    ToDer, Utf8String,
};
use std::borrow::Cow;
use std::convert::TryFrom;

impl<'a> TryFrom<Any<'a>> for &'a str {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<&'a str> {
        any.tag().assert_eq(Self::TAG)?;
        let s = Utf8String::try_from(any)?;
        match s.data {
            Cow::Borrowed(s) => Ok(s),
            Cow::Owned(_) => Err(Error::LifetimeError),
        }
    }
}

impl<'a> CheckDerConstraints for &'a str {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'a> Tagged for &'a str {
    const TAG: Tag = Tag::Utf8String;
}

impl<'a> ToDer for &'a str {
    fn to_der_len(&self) -> Result<usize> {
        let header = Header::new(
            Class::Universal,
            0,
            Self::TAG,
            Length::Definite(self.as_bytes().len()),
        );
        Ok(header.to_der_len()? + self.len())
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let header = Header::new(
            Class::Universal,
            0,
            Self::TAG,
            Length::Definite(self.as_bytes().len()),
        );
        header.write_der_header(writer).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        writer.write(self.as_bytes()).map_err(Into::into)
    }
}
