use nom::Input;

use crate::*;
use core::convert::TryFrom;

/// ASN.1 `NULL` type
#[derive(Debug, PartialEq, Eq)]
pub struct Null {}

impl Default for Null {
    fn default() -> Self {
        Self::new()
    }
}

impl Null {
    pub const fn new() -> Self {
        Null {}
    }
}

impl<'a, I: Input<Item = u8>> TryFrom<Any<'a, I>> for Null {
    type Error = Error;

    fn try_from(any: Any<'a, I>) -> Result<Null> {
        TryFrom::try_from(&any)
    }
}

impl<'a, 'b, I: Input<Item = u8>> TryFrom<&'b Any<'a, I>> for Null {
    type Error = Error;

    fn try_from(any: &'b Any<'a, I>) -> Result<Null> {
        any.tag().assert_eq(Self::TAG)?;
        if !any.header.length.is_null() {
            return Err(Error::InvalidLength);
        }
        Ok(Null {})
    }
}

impl CheckDerConstraints for Null {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl DerAutoDerive for Null {}

impl Tagged for Null {
    const TAG: Tag = Tag::Null;
}

#[cfg(feature = "std")]
impl ToDer for Null {
    fn to_der_len(&self) -> Result<usize> {
        Ok(2)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        writer.write(&[0x05, 0x00]).map_err(Into::into)
    }

    fn write_der_content(&self, _writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        Ok(0)
    }
}

impl<'a, I: Input<Item = u8>> TryFrom<Any<'a, I>> for () {
    type Error = Error;

    fn try_from(any: Any<'a, I>) -> Result<()> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        if !any.header.length.is_null() {
            return Err(Error::InvalidLength);
        }
        Ok(())
    }
}

impl CheckDerConstraints for () {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl DerAutoDerive for () {}

impl Tagged for () {
    const TAG: Tag = Tag::Null;
}

#[cfg(feature = "std")]
impl ToDer for () {
    fn to_der_len(&self) -> Result<usize> {
        Ok(2)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        writer.write(&[0x05, 0x00]).map_err(Into::into)
    }

    fn write_der_content(&self, _writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        Ok(0)
    }
}
