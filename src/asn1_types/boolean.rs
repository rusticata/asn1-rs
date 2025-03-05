use nom::{number::streaming::be_u8, AsBytes};

use crate::*;

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

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Boolean
    }

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

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Boolean
    }

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
impl ToDer for Boolean {
    fn to_der_len(&self) -> Result<usize> {
        // 3 = 1 (tag) + 1 (length) + 1 (value)
        Ok(3)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        writer.write(&[Self::TAG.0 as u8, 0x01]).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let b = if self.value != 0 { 0xff } else { 0x00 };
        writer.write(&[b]).map_err(Into::into)
    }

    /// Similar to using `to_der`, but uses header without computing length value
    fn write_der_raw(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let sz = writer.write(&[Self::TAG.0 as u8, 0x01, self.value])?;
        Ok(sz)
    }
}

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

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Boolean
    }

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        <Boolean>::from_ber_content(header, input).map(|(rem, b)| (rem, b.bool()))
    }
}

impl<'i> DerParser<'i> for bool {
    type Error = BerError<Input<'i>>;

    fn check_tag(tag: Tag) -> bool {
        tag == Tag::Boolean
    }

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
impl ToDer for bool {
    fn to_der_len(&self) -> Result<usize> {
        // 3 = 1 (tag) + 1 (length) + 1 (value)
        Ok(3)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        writer.write(&[Self::TAG.0 as u8, 0x01]).map_err(Into::into)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let b = if *self { 0xff } else { 0x00 };
        writer.write(&[b]).map_err(Into::into)
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io;
    use std::io::Write;

    use crate::{Length, Primitive, Tag, ToBer};

    impl ToBer for bool {
        type Encoder = Primitive<bool, { Tag::Boolean.0 }>;

        fn content_len(&self) -> Length {
            Length::Definite(1)
        }

        fn write_content<W: Write>(&self, target: &mut W) -> Result<usize, io::Error> {
            let value = if *self { 0xff } else { 0x00 };
            target.write(&[value])
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
}
