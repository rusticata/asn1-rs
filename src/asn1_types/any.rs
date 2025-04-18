use crate::ber::*;
use crate::debug::trace_input;
use crate::Input;
use crate::*;
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use core::convert::TryFrom;
use nom::bytes::streaming::take;

/// The `Any` object is not strictly an ASN.1 type, but holds a generic description of any object
/// that could be encoded.
///
/// It contains a header, and a reference to the object content.
///
/// Note: this type is only provided in **borrowed** version (*i.e.* it cannot own the inner data).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Any<'a> {
    /// The object header
    pub header: Header<'a>,
    /// The object contents
    pub data: Input<'a>,
}

impl<'a> Any<'a> {
    /// Create a new `Any` from BER/DER header and content
    #[inline]
    pub const fn new(header: Header<'a>, data: Input<'a>) -> Self {
        Any { header, data }
    }

    /// Create a new `Any` from BER/DER header and content
    #[inline]
    pub const fn from_header_and_data(header: Header<'a>, data: &'a [u8]) -> Self {
        Any {
            header,
            data: Input::from_slice(data),
        }
    }

    /// Create a new `Any` from a tag, and BER/DER content
    #[inline]
    pub const fn from_tag_and_data(tag: Tag, data: Input<'a>) -> Self {
        let constructed = matches!(tag, Tag::Sequence | Tag::Set);
        Any {
            header: Header {
                tag,
                constructed,
                class: Class::Universal,
                length: Length::Definite(data.len()),
                raw_tag: None,
                raw_header: None,
            },
            data,
        }
    }

    /// Return the `Class` of this object
    #[inline]
    pub const fn class(&self) -> Class {
        self.header.class
    }

    /// Update the class of the current object
    #[inline]
    pub fn with_class(self, class: Class) -> Self {
        Any {
            header: self.header.with_class(class),
            ..self
        }
    }

    /// Return the `Tag` of this object
    #[inline]
    pub const fn tag(&self) -> Tag {
        self.header.tag
    }

    /// Update the tag of the current object
    #[inline]
    pub fn with_tag(self, tag: Tag) -> Self {
        Any {
            header: self.header.with_tag(tag),
            data: self.data,
        }
    }

    /// Parse a BER value and apply the provided parsing function to content
    ///
    /// After parsing, the sequence object and header are discarded.
    pub fn from_ber_and_then<F, T, E>(
        class: Class,
        tag: u32,
        bytes: &'a [u8],
        op: F,
    ) -> ParseResult<'a, T, E>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<'a, T, E>,
        E: From<Error>,
    {
        let (rem, any): (_, Any<'a>) = Any::from_ber(bytes).map_err(Err::convert)?;
        any.tag()
            .assert_eq(Tag(tag))
            .map_err(|e| Err::Error(e.into()))?;
        any.class()
            .assert_eq(class)
            .map_err(|e| Err::Error(e.into()))?;
        let (_, res) = op(any.data.into_bytes())?;
        Ok((rem, res))
    }

    /// Parse a DER value and apply the provided parsing function to content
    ///
    /// After parsing, the sequence object and header are discarded.
    pub fn from_der_and_then<F, T, E>(
        class: Class,
        tag: u32,
        bytes: &'a [u8],
        op: F,
    ) -> ParseResult<'a, T, E>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<'a, T, E>,
        E: From<Error>,
    {
        let (rem, any) = Any::from_der(bytes).map_err(Err::convert)?;
        any.tag()
            .assert_eq(Tag(tag))
            .map_err(|e| Err::Error(e.into()))?;
        any.class()
            .assert_eq(class)
            .map_err(|e| Err::Error(e.into()))?;
        let (_, res) = op(any.data.into_bytes())?;
        Ok((rem, res))
    }

    /// Return an iterator over sub-objects
    ///
    /// This makes sense only if `Self` is constructed, but the constructed bit is
    /// _not_ checked by this function.
    pub fn iter_elements<Mode: ASN1Mode>(&'a self) -> AnyIterator<'a, BerMode> {
        AnyIterator::<BerMode>::new(self.data.clone())
    }
}

impl Any<'_> {
    /// Get the bytes representation of the *content*
    #[inline]
    pub fn as_bytes(&self) -> &'_ [u8] {
        self.data.as_ref()
    }

    #[inline]
    pub fn parse_content_ber<'a, T>(&'a self) -> ParseResult<'a, T>
    where
        T: FromBer<'a>,
    {
        T::from_ber(self.data.as_ref())
    }

    #[inline]
    pub fn parse_content_der<'a, T>(&'a self) -> ParseResult<'a, T>
    where
        T: FromDer<'a>,
    {
        T::from_der(self.data.as_ref())
    }
}

impl Any<'_> {
    /// Get the content following a BER header
    #[inline]
    pub fn parse_ber_content<'i>(
        i: Input<'i>,
        header: &'_ Header,
    ) -> IResult<Input<'i>, Input<'i>, BerError<Input<'i>>> {
        header.parse_ber_content(i)
    }

    /// Get the content following a DER header
    #[inline]
    pub fn parse_der_content<'i>(
        i: Input<'i>,
        header: &'_ Header,
    ) -> IResult<Input<'i>, Input<'i>, BerError<Input<'i>>> {
        header
            .assert_definite_inner()
            .map_err(BerError::convert(i.clone()))?;
        DerMode::get_object_content(header, i, 8)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_tryfrom_any {
    (IMPL $lft:lifetime $ty:ty ) => {
        impl<$lft> core::convert::TryFrom<$crate::Any<$lft>> for $ty {
            type Error = $crate::Error;

            fn try_from(any: $crate::Any<$lft>) -> Result<Self> {
                use $crate::BerParser;

                if !<Self as $crate::DynTagged>::accept_tag(any.tag()) {
                    Err($crate::Error::unexpected_tag(None, any.tag()).into())
                } else {
                    let (_, obj) = Self::from_ber_content(&any.header, any.data)
                        .map_err($crate::Error::from_nom_berr)?;
                    Ok(obj)
                }
            }
        }

        impl<$lft, 'b> core::convert::TryFrom<&'b $crate::Any<$lft>> for $ty {
            type Error = $crate::Error;

            fn try_from(any: &'b $crate::Any<$lft>) -> Result<Self> {
                use $crate::BerParser;

                if !<Self as $crate::DynTagged>::accept_tag(any.tag()) {
                    Err($crate::Error::unexpected_tag(None, any.tag()).into())
                } else {
                    let (_, obj) = Self::from_ber_content(&any.header, any.data.clone())
                        .map_err($crate::Error::from_nom_berr)?;
                    Ok(obj)
                }
            }
        }
    };
    // variant for lifetime present in <..> (like: OctetString<'i>)
    ($lft:lifetime @ $ty:ty) => {
        $crate::impl_tryfrom_any! {
            IMPL $lft $ty
        }
    };
    ($ty:ty) => {
        $crate::impl_tryfrom_any! {
            IMPL 'i $ty
        }
    };
}

macro_rules! impl_any_into {
    (IMPL $sname:expr, $fn_name:ident => $ty:ty, $asn1:expr) => {
        #[doc = "Attempt to convert object to `"]
        #[doc = $sname]
        #[doc = "` (ASN.1 type: `"]
        #[doc = $asn1]
        #[doc = "`)."]
        pub fn $fn_name(self) -> Result<$ty> {
            let (_, obj) =
                <$ty>::from_ber_content(&self.header, self.data).map_err(Error::from_nom_berr)?;
            Ok(obj)
        }
    };
    ($fn_name:ident => $ty:ty, $asn1:expr) => {
        impl_any_into! {
            IMPL stringify!($ty), $fn_name => $ty, $asn1
        }
    };
}

macro_rules! impl_any_as {
    (IMPL $sname:expr, $fn_name:ident => $ty:ty, $asn1:expr) => {
        #[doc = "Attempt to create ASN.1 type `"]
        #[doc = $asn1]
        #[doc = "` from this object."]
        #[doc = "\n\nNote: this method makes shallow copies of `header` and `data`."]
        #[inline]
        pub fn $fn_name(&self) -> Result<$ty> {
            // This clone is cheap
            let data = self.data.clone();
            let (_, obj) =
                <$ty>::from_ber_content(&self.header, data).map_err(Error::from_nom_berr)?;
            Ok(obj)
        }
    };
    ($fn_name:ident => $ty:ty, $asn1:expr) => {
        impl_any_as! {
            IMPL stringify!($ty), $fn_name => $ty, $asn1
        }
    };
}

impl<'a> Any<'a> {
    impl_any_into!(anysequence => AnySequence<'a>, "SEQUENCE");
    impl_any_into!(bitstring => BitString, "BIT STRING");
    impl_any_into!(bmpstring => BmpString<'a>, "BMPString");
    impl_any_into!(bool => bool, "BOOLEAN");
    impl_any_into!(boolean => Boolean, "BOOLEAN");
    impl_any_into!(embedded_pdv => EmbeddedPdv<'a>, "EMBEDDED PDV");
    impl_any_into!(enumerated => Enumerated, "ENUMERATED");
    impl_any_into!(generalizedtime => GeneralizedTime, "GeneralizedTime");
    impl_any_into!(generalstring => GeneralString<'a>, "GeneralString");
    impl_any_into!(graphicstring => GraphicString<'a>, "GraphicString");
    impl_any_into!(i8 => i8, "INTEGER");
    impl_any_into!(i16 => i16, "INTEGER");
    impl_any_into!(i32 => i32, "INTEGER");
    impl_any_into!(i64 => i64, "INTEGER");
    impl_any_into!(i128 => i128, "INTEGER");
    impl_any_into!(ia5string => Ia5String<'a>, "IA5String");
    impl_any_into!(integer => Integer<'a>, "INTEGER");
    impl_any_into!(null => Null, "NULL");
    impl_any_into!(numericstring => NumericString<'a>, "NumericString");
    impl_any_into!(objectdescriptor => ObjectDescriptor<'a>, "ObjectDescriptor");
    impl_any_into!(octetstring => OctetString<'a>, "OCTET STRING");
    impl_any_into!(oid => Oid<'a>, "OBJECT IDENTIFIER");
    impl_any_into!(real => Real, "REAL");
    /// Attempt to convert object to `Oid` (ASN.1 type: `RELATIVE-OID`).
    pub fn relative_oid(self) -> Result<Oid<'a>> {
        self.header.assert_tag(Tag::RelativeOid)?;
        let asn1 = Cow::Borrowed(self.data.into_bytes());
        Ok(Oid::new_relative(asn1))
    }
    impl_any_into!(printablestring => PrintableString<'a>, "PrintableString");
    // XXX REAL
    impl_any_into!(sequence => Sequence<'a>, "SEQUENCE");
    impl_any_into!(set => Set<'a>, "SET");
    impl_any_into!(str => &'a str, "UTF8String");
    impl_any_into!(string => String, "UTF8String");
    impl_any_into!(teletexstring => TeletexString<'a>, "TeletexString");
    impl_any_into!(u8 => u8, "INTEGER");
    impl_any_into!(u16 => u16, "INTEGER");
    impl_any_into!(u32 => u32, "INTEGER");
    impl_any_into!(u64 => u64, "INTEGER");
    impl_any_into!(u128 => u128, "INTEGER");
    impl_any_into!(universalstring => UniversalString<'a>, "UniversalString");
    impl_any_into!(utctime => UtcTime, "UTCTime");
    impl_any_into!(utf8string => Utf8String<'a>, "UTF8String");
    impl_any_into!(videotexstring => VideotexString<'a>, "VideotexString");
    impl_any_into!(visiblestring => VisibleString<'a>, "VisibleString");

    impl_any_as!(as_anysequence => AnySequence, "SEQUENCE");
    impl_any_as!(as_bitstring => BitString, "BITSTRING");
    impl_any_as!(as_bmpstring => BmpString, "BMPString");
    impl_any_as!(as_bool => bool, "BOOLEAN");
    impl_any_as!(as_boolean => Boolean, "BOOLEAN");
    impl_any_as!(as_embedded_pdv => EmbeddedPdv, "EMBEDDED PDV");
    impl_any_as!(as_endofcontent => EndOfContent, "END OF CONTENT (not a real ASN.1 type)");
    impl_any_as!(as_enumerated => Enumerated, "ENUMERATED");
    impl_any_as!(as_generalizedtime => GeneralizedTime, "GeneralizedTime");
    impl_any_as!(as_generalstring => GeneralString, "GeneralString");
    impl_any_as!(as_graphicstring => GraphicString, "GraphicString");
    impl_any_as!(as_i8 => i8, "INTEGER");
    impl_any_as!(as_i16 => i16, "INTEGER");
    impl_any_as!(as_i32 => i32, "INTEGER");
    impl_any_as!(as_i64 => i64, "INTEGER");
    impl_any_as!(as_i128 => i128, "INTEGER");
    impl_any_as!(as_ia5string => Ia5String, "IA5String");
    impl_any_as!(as_integer => Integer, "INTEGER");
    impl_any_as!(as_null => Null, "NULL");
    impl_any_as!(as_numericstring => NumericString, "NumericString");
    impl_any_as!(as_objectdescriptor => ObjectDescriptor, "OBJECT IDENTIFIER");
    impl_any_as!(as_octetstring => OctetString, "OCTET STRING");
    impl_any_as!(as_oid => Oid, "OBJECT IDENTIFIER");
    impl_any_as!(as_real => Real, "REAL");
    /// Attempt to create ASN.1 type `RELATIVE-OID` from this object.
    pub fn as_relative_oid(&'a self) -> Result<Oid<'a>> {
        self.header.assert_tag(Tag::RelativeOid)?;
        let asn1 = Cow::Borrowed(self.data.as_bytes2());
        Ok(Oid::new_relative(asn1))
    }
    impl_any_as!(as_printablestring => PrintableString, "PrintableString");
    impl_any_as!(as_sequence => Sequence, "SEQUENCE");
    impl_any_as!(as_set => Set, "SET");
    impl_any_as!(as_str => &str, "UTF8String");
    impl_any_as!(as_string => String, "UTF8String");
    impl_any_as!(as_teletexstring => TeletexString, "TeletexString");
    impl_any_as!(as_u8 => u8, "INTEGER");
    impl_any_as!(as_u16 => u16, "INTEGER");
    impl_any_as!(as_u32 => u32, "INTEGER");
    impl_any_as!(as_u64 => u64, "INTEGER");
    impl_any_as!(as_u128 => u128, "INTEGER");
    impl_any_as!(as_universalstring => UniversalString, "UniversalString");
    impl_any_as!(as_utctime => UtcTime, "UTCTime");
    impl_any_as!(as_utf8string => Utf8String, "UTF8String");
    impl_any_as!(as_videotexstring => VideotexString, "VideotexString");
    impl_any_as!(as_visiblestring => VisibleString, "VisibleString");

    /// Attempt to create an `Option<T>` from this object.
    pub fn as_optional<'b, T>(&'b self) -> Result<Option<T>>
    where
        T: TryFrom<&'b Any<'a>, Error = Error>,
        'a: 'b,
    {
        match TryFrom::try_from(self) {
            Ok(t) => Ok(Some(t)),
            Err(Error::UnexpectedTag { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Attempt to create a tagged value (EXPLICIT) from this object.
    pub fn as_tagged_explicit<T, E, const CLASS: u8, const TAG: u32>(
        &self,
    ) -> Result<TaggedValue<T, E, Explicit, CLASS, TAG>, E>
    where
        T: FromBer<'a, E>,
        E: From<Error>,
    {
        TryFrom::try_from(self)
    }

    /// Attempt to create a tagged value (IMPLICIT) from this object.
    pub fn as_tagged_implicit<T, E, const CLASS: u8, const TAG: u32>(
        &self,
    ) -> Result<TaggedValue<T, E, Implicit, CLASS, TAG>, E>
    where
        T: TryFrom<Any<'a>, Error = E>,
        T: Tagged,
        E: From<Error>,
    {
        TryFrom::try_from(self)
    }

    /// Attempt to get value as `str`, for all known string types
    ///
    /// This function does not allocate data, so it supports all string types except
    /// `BmpString` and `UniversalString`.
    pub fn as_any_str(&self) -> Result<&str> {
        match self.tag() {
            Tag::GeneralString
            | Tag::GraphicString
            | Tag::Ia5String
            | Tag::NumericString
            | Tag::PrintableString
            | Tag::T61String
            //| Tag::UniversalString // UCS-4, cannot be converted
            | Tag::Utf8String
            | Tag::VideotexString
            | Tag::VisibleString => {
                let res = core::str::from_utf8(self.data.as_bytes2())?;
                Ok(res)
            }
            _ => Err(Error::BerTypeError),
        }
    }

    /// Attempt to get value as `String`, for all known string types
    ///
    /// This function allocates data
    pub fn as_any_string(&self) -> Result<String> {
        match self.tag() {
            Tag::GeneralString
            | Tag::GraphicString
            | Tag::Ia5String
            | Tag::NumericString
            | Tag::PrintableString
            | Tag::T61String
            | Tag::Utf8String
            | Tag::VideotexString
            | Tag::VisibleString => {
                let res = core::str::from_utf8(self.data.as_bytes2())?;
                Ok(res.to_string())
            }
            Tag::BmpString => {
                let us = BmpString::try_from(self)?;
                Ok(us.string())
            }
            Tag::UniversalString => {
                let us = UniversalString::try_from(self)?;
                Ok(us.string())
            }
            _ => Err(Error::BerTypeError),
        }
    }
}

pub(crate) fn parse_ber_any(input: Input) -> IResult<Input, Any, BerError<Input>> {
    trace_input("Any", |input| {
        //
        let (i, header) = Header::parse_ber(input)?;
        let (i, data) = BerMode::get_object_content(&header, i, MAX_RECURSION)?;
        Ok((i, Any { header, data }))
    })(input)
}

pub(crate) fn parse_der_any(input: Input) -> IResult<Input, Any, BerError<Input>> {
    trace_input("Any", |input| {
        let (i, header) = Header::parse_der(input.clone())?;
        // X.690 section 10.1: The definite form of length encoding shall be used
        header
            .length
            .assert_definite_inner()
            .map_err(BerError::convert(input))?;
        let (i, data) = DerMode::get_object_content(&header, i, MAX_RECURSION)?;
        Ok((i, Any { header, data }))
    })(input)
}

impl<'a> FromBer<'a> for Any<'a> {
    #[inline]
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        wrap_ber_parser(parse_ber_any)(bytes)
    }
}

impl<'i> BerParser<'i> for Any<'i> {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, data) = match header.length {
            Length::Definite(l) => take(l)(input)?,
            Length::Indefinite => {
                // assume entire input was parsed
                take(input.len())(input)?
            }
        };
        let any = Any {
            header: header.clone(),
            data,
        };
        Ok((rem, any))
    }
}

impl<'a> FromDer<'a> for Any<'a> {
    #[inline]
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        wrap_ber_parser(parse_der_any)(bytes)
    }
}

impl<'i> DerParser<'i> for Any<'i> {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, data) = DerMode::get_object_content(header, input, MAX_RECURSION)?;
        Ok((
            rem,
            Any {
                header: header.clone(),
                data,
            },
        ))
    }
}

impl CheckDerConstraints for Any<'_> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.length().assert_definite()?;
        // if len < 128, must use short form (10.1: minimum number of octets)
        Ok(())
    }
}

impl DerAutoDerive for Any<'_> {}

impl DynTagged for Any<'_> {
    fn class(&self) -> Class {
        self.header.class
    }

    fn constructed(&self) -> bool {
        self.header.constructed
    }

    fn tag(&self) -> Tag {
        self.tag()
    }

    fn accept_tag(_: Tag) -> bool {
        // For ANY, all tags are accepted
        true
    }
}

// impl<'a> ToStatic for Any<'a> {
//     type Owned = Any<'static>;

//     fn to_static(&self) -> Self::Owned {
//         Any {
//             header: self.header.to_static(),
//             data: Cow::Owned(self.data.to_vec()),
//         }
//     }
// }

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for Any<'_> {
        type Encoder = BerGenericEncoder;

        fn ber_content_len(&self) -> Length {
            Length::Definite(self.data.len())
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write(self.data.as_bytes2()).map_err(Into::into)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), self.constructed(), self.tag())
        }
    }

    impl_toder_from_tober!(LFT 'a, Any<'a>);
};

#[cfg(test)]
mod tests {
    use crate::*;
    use hex_literal::hex;

    #[test]
    fn methods_any() {
        let header = Header::new_simple(Tag::Integer);
        let any = Any::new(header, (&[]).into())
            .with_class(Class::ContextSpecific)
            .with_tag(Tag(0));
        assert_eq!(any.as_bytes(), &[] as &[u8]);

        let input = &hex! {"80 03 02 01 01"};
        let (_, any) = Any::from_ber(input).expect("parsing failed");

        let (_, r) = any
            .parse_content_ber::<Integer>()
            .expect("parse_ber failed");
        assert_eq!(r.as_u32(), Ok(1));
        let (_, r) = any
            .parse_content_der::<Integer>()
            .expect("parse_der failed");
        assert_eq!(r.as_u32(), Ok(1));

        let header = &any.header;
        let i: Input = (&input[2..]).into();
        let (_, content) = Any::parse_ber_content(i.clone(), header).unwrap();
        assert_eq!(content.len(), 3);
        let (_, content) = Any::parse_der_content(i.clone(), header).unwrap();
        assert_eq!(content.len(), 3);

        let (_, any) = Any::from_der(&input[2..]).unwrap();
        Any::check_constraints(&any).unwrap();
        assert_eq!(<Any as DynTagged>::tag(&any), any.tag());
        let int = any.integer().unwrap();
        assert_eq!(int.as_u16(), Ok(1));
    }

    #[test]
    fn ber_parser_any() {
        let input = &hex!("02 01 02 ff ff");
        let (rem, result) = Any::parse_ber(Input::from(input)).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        assert_eq!(result.header.tag(), Tag::Integer);
    }

    #[test]
    fn parse_ber_indefinite_any() {
        // Ok: indefinite length, empty object
        let bytes: &[u8] = &hex!("a3 80 00 00");
        let (rem, _res) = Any::parse_ber(bytes.into()).expect("parsing failed");
        assert!(rem.is_empty());
        // dbg!(&_res);

        // Ok: indefinite length, non-empty object
        let bytes: &[u8] = &hex!("a3 80 0101ff 00 00");
        let (rem, _res) = Any::parse_ber(bytes.into()).expect("parsing failed");
        assert!(rem.is_empty());
        // dbg!(&_res);

        // Fail: indefinite length should be constructed
        let bytes: &[u8] = &hex!("03 80 00 00");
        let _ = Any::parse_ber(bytes.into()).expect_err("Parsing should have failed");

        // Fail: indefinite length but no EndOfContent
        let bytes: &[u8] = &hex!("a3 80 01 02 03 04");
        let _ = Any::parse_ber(bytes.into()).expect_err("Parsing should have failed");
    }
}
