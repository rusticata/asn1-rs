use crate::*;

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

impl_tryfrom_any!(Enumerated);

impl<'i> BerParser<'i> for Enumerated {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 8.4)
        header.assert_primitive_input(&input).map_err(Err::Error)?;
        // The encoding of an enumerated value shall be that of the integer value with which it is associated.
        let (rem, res) = <u32>::from_ber_content(header, input)?;
        Ok((rem, Enumerated(res)))
    }
}

impl<'i> DerParser<'i> for Enumerated {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 8.4)
        header.assert_primitive_input(&input).map_err(Err::Error)?;
        // The encoding of an enumerated value shall be that of the integer value with which it is associated.
        let (rem, res) = <u32>::from_der_content(header, input)?;
        Ok((rem, Enumerated(res)))
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, DerParser, Enumerated, Input};

    #[test]
    fn parse_ber_enum() {
        // Ok: expected data
        let input = Input::from_slice(&hex!("0a0102"));
        let (rem, res) = <Enumerated>::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(res.0, 2);

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <Enumerated>::parse_ber(input).expect_err("wrong tag");
    }

    #[test]
    fn parse_der_enum() {
        // Ok: expected data
        let input = Input::from_slice(&hex!("0a0102"));
        let (rem, res) = <Enumerated>::parse_der(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(res.0, 2);

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <Enumerated>::parse_der(input).expect_err("wrong tag");

        // Fail: non-canonical encoding of integer
        let input = Input::from_slice(&hex!("0a02ffff"));
        let _ = <Enumerated>::parse_ber(input).expect_err("non-canonical encoding");
    }
}
