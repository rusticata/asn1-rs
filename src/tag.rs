use crate::{Error, Result};
use alloc::string::ToString;
use rusticata_macros::newtype_enum;

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
    TeletexString = 0x14,
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

    pub fn invalid_value(&self, msg: &str) -> Error {
        Error::InvalidValue {
            tag: *self,
            msg: msg.to_string(),
        }
    }
}

impl From<u32> for Tag {
    fn from(v: u32) -> Self {
        Tag(v)
    }
}
