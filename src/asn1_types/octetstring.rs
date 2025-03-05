use crate::*;
use alloc::borrow::Cow;
use core::fmt::Debug;
use nom::Input as _;

pub(crate) const OCTETSTRING_MAX_RECURSION: usize = 5;

//---- OctetString

/// ASN.1 `OCTETSTRING` type
///
/// This objects implements Copy-On-Write over data:
/// - when parsing primitive form, parser will return a shared object
/// - when parsing primitive form, parser must allocate memory
///
/// This type supports constructed objects, but all data segments are appended during parsing
/// (_i.e_ object structure is not kept after parsing).
#[derive(Debug, Default, PartialEq, Eq)]
pub struct OctetString<'a> {
    data: Cow<'a, [u8]>,
}

impl<'a> OctetString<'a> {
    pub const fn new(s: &'a [u8]) -> Self {
        OctetString {
            data: Cow::Borrowed(s),
        }
    }

    /// Get the bytes representation of the *content*
    pub fn as_cow(&'a self) -> &'a Cow<'a, [u8]> {
        &self.data
    }

    /// Get the bytes representation of the *content*
    pub fn into_cow(self) -> Cow<'a, [u8]> {
        self.data
    }
}

impl AsRef<[u8]> for OctetString<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl<'a> From<&'a [u8]> for OctetString<'a> {
    fn from(b: &'a [u8]) -> Self {
        OctetString {
            data: Cow::Borrowed(b),
        }
    }
}

impl_tryfrom_any!('i @ OctetString<'i>);

impl<'i> BerParser<'i> for OctetString<'i> {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall either be primitive or constructed (X.690: 8.6.1)
        if !header.constructed() {
            let (rem, data) = input.take_split(input.len());
            Ok((
                rem,
                OctetString {
                    data: Cow::Borrowed(data.as_bytes2()),
                },
            ))
        } else {
            parse_ber_segmented(header, input, OCTETSTRING_MAX_RECURSION)
        }
    }
}

impl<'i> DerParser<'i> for OctetString<'i> {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        <OctetString>::from_ber_content(header, input)
    }
}

impl CheckDerConstraints for OctetString<'_> {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl DerAutoDerive for OctetString<'_> {}

impl Tagged for OctetString<'_> {
    const TAG: Tag = Tag::OctetString;
}

impl Appendable for OctetString<'_> {
    fn append(&mut self, other: &mut Self) {
        match &mut self.data {
            Cow::Borrowed(data) => {
                let mut v = data.to_vec();
                v.extend_from_slice(&other.data);
                self.data = Cow::Owned(v);
            }
            Cow::Owned(s) => s.extend_from_slice(&other.data),
        };
    }
}

#[cfg(feature = "std")]
impl ToDer for OctetString<'_> {
    fn to_der_len(&self) -> Result<usize> {
        let sz = self.data.len();
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
            Length::Definite(self.data.len()),
        );
        header.write_der_header(writer)
    }

    fn write_der_content(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        writer.write(&self.data).map_err(Into::into)
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for OctetString<'_> {
        type Encoder = Primitive<Self, { Tag::OctetString.0 }>;

        fn content_len(&self) -> Length {
            Length::Definite(self.data.len())
        }

        fn write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write(&self.data).map_err(Into::into)
        }
    }
};

//---- &[u8]

impl_tryfrom_any!('i @ &'i [u8]);

impl<'i> BerParser<'i> for &'i [u8] {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall either be primitive or constructed (X.690: 8.6.1)
        // However, we are implementing for a shared slice, so it cannot use constructed form
        // (which requires allocation)
        if header.is_constructed() {
            return Err(BerError::nom_err_input(&input, InnerError::LifetimeError))?;
        }
        let (rem, data) = input.take_split(input.len());
        Ok((rem, data.as_bytes2()))
    }
}

impl<'i> DerParser<'i> for &'i [u8] {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        Self::from_ber_content(header, input)
    }
}

impl CheckDerConstraints for &'_ [u8] {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        Ok(())
    }
}

impl DerAutoDerive for &'_ [u8] {}

impl Tagged for &'_ [u8] {
    const TAG: Tag = Tag::OctetString;
}

#[cfg(feature = "std")]
impl ToDer for &'_ [u8] {
    fn to_der_len(&self) -> Result<usize> {
        let header = Header::new(
            Class::Universal,
            false,
            Self::TAG,
            Length::Definite(self.len()),
        );
        Ok(header.to_der_len()? + self.len())
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
        writer.write(self).map_err(Into::into)
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for &'_ [u8] {
        type Encoder = Primitive<Self, { Tag::OctetString.0 }>;

        fn content_len(&self) -> Length {
            Length::Definite(self.len())
        }

        fn write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write(self).map_err(Into::into)
        }
    }
};

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;

    use hex_literal::hex;

    use crate::{BerParser, Input, OctetString};

    #[test]
    fn parse_ber_octetstring() {
        // coverage
        let s = OctetString::new(b"1234");
        assert_eq!(s.as_cow().len(), 4);
        assert_eq!(s.into_cow(), Cow::Borrowed(b"1234"));
        //
        let input = &hex!("04 05 41 41 41 41 41");
        let (rem, result) = OctetString::parse_ber(Input::from(input)).expect("parsing failed");
        assert_eq!(result.as_ref(), b"AAAAA");
        assert!(rem.is_empty());
        //
        let (rem, result) = <&[u8]>::parse_ber(Input::from(input)).expect("parsing failed");
        assert_eq!(result, b"AAAAA");
        assert!(rem.is_empty());
    }

    #[test]
    fn parse_ber_octetstring_constructed() {
        let bytes = &hex!(
            "24 80\
   04 08 0011223344556677\
   04 08 8899AABBCCDDEEFF\
00 00"
        );
        let expected = &hex!("00112233445566778899AABBCCDDEEFF");

        let (rem, res) =
            OctetString::parse_ber(Input::from(bytes)).expect("parsing as OctetString");
        assert!(rem.is_empty());
        assert!(matches!(res.data, Cow::Owned(_)));
        assert_eq!(res.as_ref(), expected);

        // Fail: parsing as &[u8] can't be done, it would require an allocation
        let _ = <&[u8]>::parse_ber(Input::from(bytes)).expect_err("parsing as slice");
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{OctetString, ToBer};

        #[test]
        fn tober_octetstring() {
            let i = OctetString::new(&hex!("01020304"));
            let mut v: Vec<u8> = Vec::new();
            i.encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"0404 01020304"});
        }
    }
}
