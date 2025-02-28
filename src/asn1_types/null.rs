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

impl<'a> TryFrom<Any<'a>> for Null {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Null> {
        TryFrom::try_from(&any)
    }
}

impl<'a, 'b> TryFrom<&'b Any<'a>> for Null {
    type Error = Error;

    fn try_from(any: &'b Any<'a>) -> Result<Null> {
        any.tag().assert_eq(Self::TAG)?;
        if !any.header.length.is_null() {
            return Err(Error::InvalidLength);
        }
        Ok(Null {})
    }
}

impl<'i> BerParser<'i> for Null {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Null
    }

    fn from_ber_content(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 8.8.1)
        header.assert_primitive_input(&input).map_err(Err::Error)?;
        // Content octets shall not contain any octets (X.690: 8.8.2)
        if !header.length.is_null() {
            return Err(Err::Error(BerError::new(input, InnerError::InvalidLength)));
        }
        Ok((input, Null {}))
    }
}

impl<'i> DerParser<'i> for Null {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Null
    }

    fn from_der_content(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        // DER parser is the same as BER
        <Null>::from_ber_content(input, header)
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

impl<'a> TryFrom<Any<'a>> for () {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<()> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_primitive()?;
        if !any.header.length.is_null() {
            return Err(Error::InvalidLength);
        }
        Ok(())
    }
}

impl<'i> BerParser<'i> for () {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Null
    }

    fn from_ber_content(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        <Null>::from_ber_content(input, header).map(|(rem, _)| (rem, ()))
    }
}

impl<'i> DerParser<'i> for () {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Null
    }

    fn from_der_content(input: Input<'i>, header: Header<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        <Null>::from_der_content(input, header).map(|(rem, _)| (rem, ()))
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, DerParser, Input, Null};

    #[test]
    fn parse_ber_null() {
        // Ok: expected data
        let input = Input::from_slice(&hex!("0500"));
        let (rem, res) = <Null>::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(res, Null {});

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <Null>::parse_ber(input).expect_err("wrong tag");

        // Fail: non-null content
        let input = Input::from_slice(&hex!("050100"));
        let _ = <Null>::parse_ber(input).expect_err("non-null content");
    }

    #[test]
    fn parse_der_null() {
        // Ok: expected data
        let input = Input::from_slice(&hex!("0500"));
        let (rem, res) = <Null>::parse_der(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(res, Null {});

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <Null>::parse_der(input).expect_err("wrong tag");

        // Fail: non-null content
        let input = Input::from_slice(&hex!("050100"));
        let _ = <Null>::parse_der(input).expect_err("non-null content");
    }
}
