use crate::ber::bytes_to_u64;
use crate::{
    Any, CheckDerConstraints, Class, Error, Header, Integer, Length, Result, SerializeResult, Tag,
    Tagged, ToDer,
};
use std::convert::TryFrom;

#[derive(Debug, PartialEq)]
pub struct Enumerated(pub u32);

impl Enumerated {
    pub const fn new(value: u32) -> Self {
        Enumerated(value)
    }
}

impl<'a> TryFrom<Any<'a>> for Enumerated {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Enumerated> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        let res_u64 = bytes_to_u64(&any.data)?;
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

impl Tagged for Enumerated {
    const TAG: Tag = Tag::Enumerated;
}

impl ToDer for Enumerated {
    fn to_der_len(&self) -> Result<usize> {
        Integer::from(self.0).to_der_len()
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let len = Integer::from(self.0).to_der_len()?;
        let header = Header::new(Class::Universal, 0, Self::TAG, Length::Definite(len));
        header.write_der_header(writer).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let int = Integer::from(self.0);
        int.write_der_content(writer).map_err(Into::into)
    }
}
