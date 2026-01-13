use nom::{number::streaming::be_u8, AsBytes};

use crate::*;

//---- Boolean

/// ASN.1 `BOOLEAN` type
///
/// BER objects consider any non-zero value as `true`, and `0` as `false`.
///
/// DER objects must use value `0x0` (`false`) or `0xff` (`true`).
#[derive(Debug, PartialEq, Eq)]
pub struct Boolean {
    pub value: u8,
}

impl Boolean {
    /// `BOOLEAN` object for value `false`
    pub const FALSE: Boolean = Boolean::new(0);
    /// `BOOLEAN` object for value `true`
    pub const TRUE: Boolean = Boolean::new(0xff);

    /// Create a new `Boolean` from the provided logical value.
    #[inline]
    pub const fn new(value: u8) -> Self {
        Boolean { value }
    }

    /// Return the `bool` value from this object.
    #[inline]
    pub const fn bool(&self) -> bool {
        self.value != 0
    }
}

impl_tryfrom_any!(Boolean);

impl<'i> BerParser<'i> for Boolean {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 8.2.1)
        header.assert_primitive_input(&input).map_err(Err::Error)?;
        // The contents octets shall consist of a single octet (X.690: 8.2.1)
        if header.length != Length::Definite(1) {
            return Err(BerError::nom_err(input, InnerError::InvalidLength));
        }
        let (rem, value) = be_u8(input)?;
        Ok((rem, Boolean { value }))
    }
}

impl<'i> DerParser<'i> for Boolean {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, b) = Boolean::from_ber_content(header, input)?;
        if !(b.value == 0 || b.value == 0xff) {
            let e = DerConstraint::InvalidBoolean;
            return Err(BerError::nom_err_input(
                &rem,
                InnerError::DerConstraintFailed(e),
            ));
        }
        Ok((rem, b))
    }
}

impl CheckDerConstraints for Boolean {
    fn check_constraints(any: &Any) -> Result<()> {
        let c = any.data.as_bytes()[0];
        // X.690 section 11.1
        if !(c == 0 || c == 0xff) {
            return Err(Error::DerConstraintFailed(DerConstraint::InvalidBoolean));
        }
        Ok(())
    }
}

impl DerAutoDerive for Boolean {}

impl Tagged for Boolean {
    const TAG: Tag = Tag::Boolean;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for Boolean {
        type Encoder = Primitive<{ Tag::Boolean.0 }>;

        fn ber_content_len(&self) -> Length {
            Length::Definite(1)
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write_all(&[self.value])?;
            Ok(1)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), self.constructed(), self.tag())
        }
    }

    impl ToDer for Boolean {
        type Encoder = Primitive<{ Tag::Boolean.0 }>;

        fn der_content_len(&self) -> Length {
            Length::Definite(1)
        }

        fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let value = if self.value != 0 { 0xff } else { 0x00 };
            target.write_all(&[value])?;
            Ok(1)
        }

        fn der_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), self.constructed(), self.tag())
        }
    }
};

//---- bool

impl_tryfrom_any!(bool);

impl CheckDerConstraints for bool {
    fn check_constraints(any: &Any) -> Result<()> {
        let c = any.data.as_bytes()[0];
        // X.690 section 11.1
        if !(c == 0 || c == 0xff) {
            return Err(Error::DerConstraintFailed(DerConstraint::InvalidBoolean));
        }
        Ok(())
    }
}

impl<'i> BerParser<'i> for bool {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        <Boolean>::from_ber_content(header, input).map(|(rem, b)| (rem, b.bool()))
    }
}

impl<'i> DerParser<'i> for bool {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        <Boolean>::from_der_content(header, input).map(|(rem, b)| (rem, b.bool()))
    }
}

impl DerAutoDerive for bool {}

impl Tagged for bool {
    const TAG: Tag = Tag::Boolean;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for bool {
        type Encoder = Primitive<{ Tag::Boolean.0 }>;

        fn ber_content_len(&self) -> Length {
            Length::Definite(1)
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let value = if *self { 0xff } else { 0x00 };
            target.write_all(&[value])?;
            Ok(1)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl ToDer for bool {
        type Encoder = Primitive<{ Tag::Boolean.0 }>;

        fn der_content_len(&self) -> Length {
            Length::Definite(1)
        }

        fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let value = if *self { 0xff } else { 0x00 };
            target.write_all(&[value])?;
            Ok(1)
        }

        fn der_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), self.constructed(), self.tag())
        }
    }
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, Boolean, DerParser, Input};

    #[test]
    fn parse_ber_bool() {
        //--- Boolean
        // Ok: expected data
        let input = Input::from_slice(&hex!("0101ff"));
        let (rem, res) = <Boolean>::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res.bool());

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <Boolean>::parse_ber(input).expect_err("wrong tag");

        // Fail: content with invalid length
        let input = Input::from_slice(&hex!("0102ffff"));
        let _ = <Boolean>::parse_ber(input).expect_err("invalid length");

        //--- bool
        // Ok: expected data
        let input = Input::from_slice(&hex!("0101ff"));
        let (rem, res) = <bool>::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res);

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <bool>::parse_ber(input).expect_err("wrong tag");

        // Fail: content with invalid length
        let input = Input::from_slice(&hex!("0102ffff"));
        let _ = <bool>::parse_ber(input).expect_err("invalid length");
    }

    #[test]
    fn parse_der_bool() {
        //--- Boolean
        // Ok: expected data
        let input = Input::from_slice(&hex!("0101ff"));
        let (rem, res) = <Boolean>::parse_der(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res.bool());

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <Boolean>::parse_der(input).expect_err("wrong tag");

        // Fail: content with invalid length
        let input = Input::from_slice(&hex!("0102ffff"));
        let _ = <Boolean>::parse_der(input).expect_err("invalid length");

        // Fail: non-canonical boolean
        let input = Input::from_slice(&hex!("010101"));
        let _ = <Boolean>::parse_der(input).expect_err("non-canonical");

        //--- bool
        // Ok: expected data
        let input = Input::from_slice(&hex!("0101ff"));
        let (rem, res) = <bool>::parse_der(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert!(res);

        // Fail: wrong tag
        let input = Input::from_slice(&hex!("0400"));
        let _ = <bool>::parse_der(input).expect_err("wrong tag");

        // Fail: content with invalid length
        let input = Input::from_slice(&hex!("0102ffff"));
        let _ = <bool>::parse_der(input).expect_err("invalid length");

        // Fail: non-canonical boolean
        let input = Input::from_slice(&hex!("010101"));
        let _ = <bool>::parse_der(input).expect_err("non-canonical");
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{Boolean, ToBer};

        #[test]
        fn tober_bool() {
            // Ok: Boolean
            let b = Boolean::TRUE;
            let mut v: Vec<u8> = Vec::new();
            b.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"0101ff"});

            // Ok: bool
            let mut v: Vec<u8> = Vec::new();
            true.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"0101ff"});
        }
    }
}
