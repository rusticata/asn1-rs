use crate::*;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use nom::Input as _;

impl_tryfrom_any!(String);

impl CheckDerConstraints for String {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl<'i> BerParser<'i> for String {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, obj) = Utf8String::from_ber_content(header, input)?;

        let s = obj.data.into_owned();
        Ok((rem, s))
    }
}

impl<'i> DerParser<'i> for String {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        let (rem, data) = input.take_split(input.len());

        match core::str::from_utf8(data.as_bytes2()) {
            Ok(s) => Ok((rem, s.to_string())),
            Err(_) => Err(BerError::nom_err_input(
                &rem,
                InnerError::StringInvalidCharset,
            )),
        }
    }
}

impl DerAutoDerive for String {}

impl Tagged for String {
    const TAG: Tag = Tag::Utf8String;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for String {
        type Encoder = Primitive<{ Tag::Utf8String.0 }>;

        fn ber_content_len(&self) -> Length {
            Length::Definite(self.len())
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write(self.as_bytes()).map_err(Into::into)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(TY String);
};

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use hex_literal::hex;

    use crate::{BerParser, DerParser, Input};

    #[test]
    fn parse_ber_string() {
        // Ok: valid input
        let input = &hex!("0c 03 31 32 33");
        let (rem, result) = <String>::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result, "123");

        // Fail: wrong charset
        let input = &hex!("1C 10 00000061 00000062 00000063 00000064"); // this is UCS-4
        let _ = <String>::parse_ber(Input::from(input)).expect_err("parsing should fail");
    }

    #[test]
    fn parse_der_string() {
        // Ok: valid input
        let input = &hex!("0c 03 31 32 33");
        let (rem, result) = <String>::parse_der(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result, "123");

        // Fail: wrong charset
        let input = &hex!("1C 10 00000061 00000062 00000063 00000064"); // this is UCS-4
        let _ = <String>::parse_der(Input::from(input)).expect_err("parsing should fail");
    }
}
