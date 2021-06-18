use crate::ber::*;
use crate::der_constraint_fail_if;
use crate::error::*;
use crate::ToStatic;
use crate::{FromBer, FromDer};
use nom::bytes::streaming::take;
use rusticata_macros::newtype_enum;
use std::borrow::Cow;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct BerClassFromIntError(pub(crate) ());

/// BER Object class of tag
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Class {
    Universal = 0b00,
    Application = 0b01,
    ContextSpecific = 0b10,
    Private = 0b11,
}

/// BER/DER Tag as defined in X.680 section 8.4
///
/// X.690 doesn't specify the maximum tag size so we're assuming that people
/// aren't going to need anything more than a u32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tag(pub u32);

newtype_enum! {
impl display Tag {
    EndOfContent = 0x0,
    Boolean = 0x1,
    Integer = 0x2,
    BitString = 0x3,
    OctetString = 0x4,
    Null = 0x05,
    Oid = 0x06,
    ObjDescriptor = 0x07,
    External = 0x08,
    RealType = 0x09,
    Enumerated = 0xa,
    EmbeddedPdv = 0xb,
    Utf8String = 0xc,
    RelativeOid = 0xd,

    Sequence = 0x10,
    Set = 0x11,
    NumericString = 0x12,
    PrintableString = 0x13,
    T61String = 0x14,
    VideotexString = 0x15,

    Ia5String = 0x16,
    UtcTime = 0x17,
    GeneralizedTime = 0x18,

    GraphicString = 25, // 0x19
    VisibleString = 26, // 0x1a
    GeneralString = 27, // 0x1b

    UniversalString = 0x1c,
    BmpString = 0x1e,

    Invalid = 0xff,
}
}

/// BER Object Length
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Length {
    /// Definite form (X.690 8.1.3.3)
    Definite(usize),
    /// Indefinite form (X.690 8.1.3.6)
    Indefinite,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Header<'a> {
    pub class: Class,
    pub structured: u8,
    pub tag: Tag,
    pub length: Length,
    pub(crate) raw_tag: Option<Cow<'a, [u8]>>,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Class::Universal => "UNIVERSAL",
            Class::Application => "APPLICATION",
            Class::ContextSpecific => "CONTEXT-SPECIFIC",
            Class::Private => "PRIVATE",
        };
        write!(f, "{}", s)
    }
}

impl TryFrom<u8> for Class {
    type Error = BerClassFromIntError;

    #[inline]
    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            0b00 => Ok(Class::Universal),
            0b01 => Ok(Class::Application),
            0b10 => Ok(Class::ContextSpecific),
            0b11 => Ok(Class::Private),
            _ => Err(BerClassFromIntError(())),
        }
    }
}

impl Tag {
    pub const fn assert_eq(&self, tag: Tag) -> Result<()> {
        if self.0 == tag.0 {
            Ok(())
        } else {
            Err(Error::UnexpectedTag {
                expected: Some(tag),
                actual: *self,
            })
        }
    }
}

impl From<u32> for Tag {
    fn from(v: u32) -> Self {
        Tag(v)
    }
}

impl Length {
    /// Return true if length is definite and equal to 0
    #[inline]
    pub fn is_null(&self) -> bool {
        *self == Length::Definite(0)
    }

    /// Get length of primitive object
    #[inline]
    pub fn definite(&self) -> Result<usize> {
        match self {
            Length::Definite(sz) => Ok(*sz),
            Length::Indefinite => Err(Error::IndefiniteLengthUnexpected),
        }
    }

    /// Return true if length is definite
    #[inline]
    pub const fn is_definite(&self) -> bool {
        matches!(self, Length::Definite(_))
    }

    /// Return error if length is not definite
    #[inline]
    pub const fn assert_definite(&self) -> Result<()> {
        match self {
            Length::Definite(_) => Ok(()),
            Length::Indefinite => Err(Error::IndefiniteLengthUnexpected),
        }
    }
}

impl From<usize> for Length {
    fn from(l: usize) -> Self {
        Length::Definite(l)
    }
}

impl<'a> Header<'a> {
    /// Build a new BER header
    pub const fn new(class: Class, structured: u8, tag: Tag, length: Length) -> Self {
        Header {
            tag,
            structured,
            class,
            length,
            raw_tag: None,
        }
    }

    /// Update header to add reference to raw tag
    #[inline]
    pub fn with_raw_tag(self, raw_tag: Option<Cow<'a, [u8]>>) -> Self {
        Header { raw_tag, ..self }
    }

    /// Test if object is primitive
    #[inline]
    pub const fn is_primitive(&self) -> bool {
        self.structured == 0
    }

    /// Test if object is constructed
    #[inline]
    pub const fn is_constructed(&self) -> bool {
        self.structured == 1
    }

    /// Return error if class is not the expected class
    #[inline]
    pub const fn assert_class(&self, class: Class) -> Result<()> {
        if self.class as u8 == class as u8 {
            Ok(())
        } else {
            Err(Error::UnexpectedClass(class))
        }
    }

    /// Return error if tag is not the expected tag
    #[inline]
    pub const fn assert_tag(&self, tag: Tag) -> Result<()> {
        self.tag.assert_eq(tag)
    }

    /// Return error if object is not primitive
    #[inline]
    pub const fn assert_primitive(&self) -> Result<()> {
        if self.is_primitive() {
            Ok(())
        } else {
            Err(Error::ConstructUnexpected)
        }
    }

    /// Return error if object is primitive
    #[inline]
    pub const fn assert_constructed(&self) -> Result<()> {
        if !self.is_primitive() {
            Ok(())
        } else {
            Err(Error::ConstructExpected)
        }
    }
}

impl<'a> ToStatic for Header<'a> {
    type Owned = Header<'static>;

    fn to_static(&self) -> Self::Owned {
        let raw_tag: Option<Cow<'static, [u8]>> =
            self.raw_tag.as_ref().map(|b| Cow::Owned(b.to_vec()));
        Header {
            tag: self.tag,
            structured: self.structured,
            class: self.class,
            length: self.length,
            raw_tag,
        }
    }
}

impl<'a> FromBer<'a> for Header<'a> {
    fn from_ber(bytes: &'a [u8]) -> ParseResult<Self> {
        let (i1, el) = parse_identifier(bytes)?;
        let class = match Class::try_from(el.0) {
            Ok(c) => c,
            Err(_) => unreachable!(), // Cannot fail, we have read exactly 2 bits
        };
        let (i2, len) = parse_ber_length_byte(i1)?;
        let (i3, len) = match (len.0, len.1) {
            (0, l1) => {
                // Short form: MSB is 0, the rest encodes the length (which can be 0) (8.1.3.4)
                (i2, Length::Definite(usize::from(l1)))
            }
            (_, 0) => {
                // Indefinite form: MSB is 1, the rest is 0 (8.1.3.6)
                // If encoding is primitive, definite form shall be used (8.1.3.2)
                if el.1 == 0 {
                    return Err(nom::Err::Error(Error::ConstructExpected));
                }
                (i2, Length::Indefinite)
            }
            (_, l1) => {
                // if len is 0xff -> error (8.1.3.5)
                if l1 == 0b0111_1111 {
                    return Err(::nom::Err::Error(Error::InvalidLength));
                }
                let (i3, llen) = take(l1)(i2)?;
                match bytes_to_u64(llen) {
                    Ok(l) => {
                        let l =
                            usize::try_from(l).or(Err(::nom::Err::Error(Error::InvalidLength)))?;
                        (i3, Length::Definite(l))
                    }
                    Err(_) => {
                        return Err(::nom::Err::Error(Error::InvalidLength));
                    }
                }
            }
        };
        let hdr = Header::new(class, el.1, Tag(el.2), len).with_raw_tag(Some(el.3.into()));
        Ok((i3, hdr))
    }
}

impl<'a> FromDer<'a> for Header<'a> {
    fn from_der(bytes: &'a [u8]) -> ParseResult<Self> {
        let (i1, el) = parse_identifier(bytes)?;
        let class = match Class::try_from(el.0) {
            Ok(c) => c,
            Err(_) => unreachable!(), // Cannot fail, we have read exactly 2 bits
        };
        let (i2, len) = parse_ber_length_byte(i1)?;
        let (i3, len) = match (len.0, len.1) {
            (0, l1) => {
                // Short form: MSB is 0, the rest encodes the length (which can be 0) (8.1.3.4)
                (i2, Length::Definite(usize::from(l1)))
            }
            (_, 0) => {
                // Indefinite form is not allowed in DER (10.1)
                return Err(::nom::Err::Error(Error::IndefiniteLengthUnexpected));
            }
            (_, l1) => {
                // if len is 0xff -> error (8.1.3.5)
                if l1 == 0b0111_1111 {
                    return Err(::nom::Err::Error(Error::InvalidLength));
                }
                // DER(9.1) if len is 0 (indefinite form), obj must be constructed
                der_constraint_fail_if!(&i[1..], len.1 == 0 && el.1 != 1);
                let (i3, llen) = take(l1)(i2)?;
                match bytes_to_u64(llen) {
                    Ok(l) => {
                        // DER: should have been encoded in short form (< 127)
                        der_constraint_fail_if!(i, l < 127);
                        let l =
                            usize::try_from(l).or(Err(::nom::Err::Error(Error::InvalidLength)))?;
                        (i3, Length::Definite(l))
                    }
                    Err(_) => {
                        return Err(::nom::Err::Error(Error::InvalidLength));
                    }
                }
            }
        };
        let hdr = Header::new(class, el.1, Tag(el.2), len).with_raw_tag(Some(el.3.into()));
        Ok((i3, hdr))
    }
}
