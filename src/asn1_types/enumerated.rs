use nom::Input;

use crate::*;
use core::convert::TryFrom;

/// ASN.1 `ENUMERATED` type
///
/// # Limitations
///
/// Supported values are limited to 0 .. 2^32
#[derive(Debug, PartialEq, Eq)]
pub struct Enumerated(pub u32);

impl Enumerated {
    pub const fn new(value: u32) -> Self {
        Enumerated(value)
    }
}

impl<'a, I: Input<Item = u8>> TryFrom<Any<'a, I>> for Enumerated {
    type Error = Error;

    fn try_from(any: Any<'a, I>) -> Result<Enumerated> {
        TryFrom::try_from(&any)
    }
}

impl<'a, 'b, I: Input<Item = u8>> TryFrom<&'b Any<'a, I>> for Enumerated {
    type Error = Error;

    fn try_from(any: &'b Any<'a, I>) -> Result<Enumerated> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        let res_u64 = bytes_to_u64_g(any.data.clone())?;
        if res_u64 > (<u32>::MAX as u64) {
            return Err(Error::IntegerTooLarge);
        }
        let value = res_u64 as u32;
        Ok(Enumerated(value))
    }
}

impl CheckDerConstraints for Enumerated {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl DerAutoDerive for Enumerated {}

impl Tagged for Enumerated {
    const TAG: Tag = Tag::Enumerated;
}

#[cfg(feature = "std")]
impl ToDer for Enumerated {
    fn to_der_len(&self) -> Result<usize> {
        Integer::from(self.0).to_der_len()
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let i = Integer::from(self.0);
        let len = i.data.len();
        let header = Header::new(Class::Universal, false, Self::TAG, Length::Definite(len));
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let int = Integer::from(self.0);
        int.write_der_content(writer)
    }
}
