use crate::*;
use nom::Input as _;

impl_tryfrom_any!('i @ &'i str);

impl CheckDerConstraints for &'_ str {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'i> BerParser<'i> for &'i str {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall either be primitive or constructed (X.690: 8.20)
        // However, we are implementing for a shared slice, so it cannot use constructed form
        // (which requires allocation)
        if header.is_constructed() {
            return Err(BerError::nom_err_input(&input, InnerError::LifetimeError))?;
        }

        let (rem, data) = input.take_split(input.len());

        match core::str::from_utf8(data.as_bytes2()) {
            Ok(s) => Ok((rem, s)),
            Err(_) => Err(BerError::nom_err_input(
                &rem,
                InnerError::StringInvalidCharset,
            )),
        }
    }
}

impl<'i> DerParser<'i> for &'i str {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        let (rem, data) = input.take_split(input.len());

        match core::str::from_utf8(data.as_bytes2()) {
            Ok(s) => Ok((rem, s)),
            Err(_) => Err(BerError::nom_err_input(
                &rem,
                InnerError::StringInvalidCharset,
            )),
        }
    }
}

impl DerAutoDerive for &'_ str {}

impl Tagged for &'_ str {
    const TAG: Tag = Tag::Utf8String;
}

#[cfg(feature = "std")]
impl ToDer for &'_ str {
    fn to_der_len(&self) -> Result<usize> {
        let sz = self.len();
        if sz < 127 {
            // 1 (class+tag) + 1 (length) + len
            Ok(2 + sz)
        } else {
            // 1 (class+tag) + n (length) + len
            let n = Length::Definite(sz).to_der_len()?;
            Ok(1 + n + sz)
        }
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let header = Header::new(
            Class::Universal,
            false,
            Self::TAG,
            Length::Definite(self.len()),
        );
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        writer.write(self.as_bytes()).map_err(Into::into)
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io;
    use std::io::Write;

    impl ToBer for &'_ str {
        type Encoder = Primitive<Self, { Tag::Utf8String.0 }>;

        fn content_len(&self) -> Length {
            Length::Definite(self.len())
        }

        fn write_content<W: Write>(&self, target: &mut W) -> Result<usize, io::Error> {
            target.write(self.as_bytes())
        }
    }
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, DerParser, Input};

    #[test]
    fn parse_ber_str() {
        // Ok: valid input
        let input = &hex!("0c 03 31 32 33");
        let (rem, result) = <&str>::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result, "123");

        // Fail: wrong charset
        let input = &hex!("1C 10 00000061 00000062 00000063 00000064"); // this is UCS-4
        let _ = <&str>::parse_ber(Input::from(input)).expect_err("parsing should fail");
    }

    #[test]
    fn parse_der_str() {
        // Ok: valid input
        let input = &hex!("0c 03 31 32 33");
        let (rem, result) = <&str>::parse_der(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result, "123");

        // Fail: wrong charset
        let input = &hex!("1C 10 00000061 00000062 00000063 00000064"); // this is UCS-4
        let _ = <&str>::parse_der(Input::from(input)).expect_err("parsing should fail");
    }
}
