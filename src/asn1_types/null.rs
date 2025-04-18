use crate::*;

//---- Null

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

impl_tryfrom_any!(Null);

impl<'i> BerParser<'i> for Null {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
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

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // DER parser is the same as BER
        <Null>::from_ber_content(header, input)
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
const _: () = {
    use std::io::Write;

    impl ToBer for Null {
        type Encoder = Primitive<{ Tag::Null.0 }>;

        fn ber_content_len(&self) -> Length {
            Length::Definite(0)
        }

        fn ber_write_content<W: Write>(&self, _: &mut W) -> SerializeResult<usize> {
            Ok(0)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(TY Null);
};

//---- ()

impl_tryfrom_any!(());

impl<'i> BerParser<'i> for () {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        <Null>::from_ber_content(header, input).map(|(rem, _)| (rem, ()))
    }
}

impl<'i> DerParser<'i> for () {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        <Null>::from_der_content(header, input).map(|(rem, _)| (rem, ()))
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
const _: () = {
    use std::io::Write;

    impl ToBer for () {
        type Encoder = Primitive<{ Tag::Null.0 }>;

        fn ber_content_len(&self) -> Length {
            Length::Definite(0)
        }

        fn ber_write_content<W: Write>(&self, _: &mut W) -> SerializeResult<usize> {
            Ok(0)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(TY());
};

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
