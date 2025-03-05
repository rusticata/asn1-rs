use crate::ber::*;
use crate::der_constraint_fail_if;
use crate::error::*;
use crate::DerParser;
#[cfg(feature = "std")]
use crate::ToDer;
use crate::{BerMode, Class, DerMode, DynTagged, FromBer, FromDer, Length, Tag, ToStatic};
use crate::{BerParser, Input};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::convert::TryFrom;
use core::hash::Hash;
use nom::bytes::streaming::take;
use nom::number::streaming::be_u8;
use nom::{Err, IResult};

/// BER/DER object header (identifier and length)
#[derive(Clone, Debug)]
pub struct Header<'a> {
    /// Object class: universal, application, context-specific, or private
    pub(crate) class: Class,
    /// Constructed attribute: true if constructed, else false
    pub(crate) constructed: bool,
    /// Tag number
    pub(crate) tag: Tag,
    /// Object length: value if definite, or indefinite
    pub(crate) length: Length,

    /// Optionally, the raw encoding of the tag
    ///
    /// This is useful in some cases, where different representations of the same
    /// BER tags have different meanings (BER only)
    pub(crate) raw_tag: Option<Cow<'a, [u8]>>,
}

impl<'a> Header<'a> {
    /// Build a new BER/DER header from the provided values
    pub const fn new(class: Class, constructed: bool, tag: Tag, length: Length) -> Self {
        Header {
            tag,
            constructed,
            class,
            length,
            raw_tag: None,
        }
    }

    /// Build a new BER/DER header from the provided tag, with default values for other fields
    #[inline]
    pub const fn new_simple(tag: Tag) -> Self {
        let constructed = matches!(tag, Tag::Sequence | Tag::Set);
        Self::new(Class::Universal, constructed, tag, Length::Definite(0))
    }

    /// Set the class of this `Header`
    #[inline]
    pub fn with_class(self, class: Class) -> Self {
        Self { class, ..self }
    }

    /// Set the constructed flags of this `Header`
    #[inline]
    pub fn with_constructed(self, constructed: bool) -> Self {
        Self {
            constructed,
            ..self
        }
    }

    /// Set the tag of this `Header`
    #[inline]
    pub fn with_tag(self, tag: Tag) -> Self {
        Self { tag, ..self }
    }

    /// Set the length of this `Header`
    #[inline]
    pub fn with_length(self, length: Length) -> Self {
        Self { length, ..self }
    }

    /// Update header to add reference to raw tag
    #[inline]
    pub fn with_raw_tag(self, raw_tag: Option<Cow<'a, [u8]>>) -> Self {
        Header { raw_tag, ..self }
    }

    /// Return the class of this header.
    #[inline]
    pub const fn class(&self) -> Class {
        self.class
    }

    /// Return true if this header has the 'constructed' flag.
    #[inline]
    pub const fn constructed(&self) -> bool {
        self.constructed
    }

    /// Return the tag of this header.
    #[inline]
    pub const fn tag(&self) -> Tag {
        self.tag
    }

    /// Return the length of this header.
    #[inline]
    pub const fn length(&self) -> Length {
        self.length
    }

    /// Return the raw tag encoding, if it was stored in this object
    #[inline]
    pub fn raw_tag(&self) -> Option<&[u8]> {
        self.raw_tag.as_ref().map(|cow| cow.as_ref())
    }

    /// Test if object is primitive
    #[inline]
    pub const fn is_primitive(&self) -> bool {
        !self.constructed
    }

    /// Test if object is constructed
    #[inline]
    pub const fn is_constructed(&self) -> bool {
        self.constructed
    }

    /// Return error if class is not the expected class
    #[inline]
    pub const fn assert_class(&self, class: Class) -> Result<()> {
        self.class.assert_eq(class)
    }

    /// Return error if tag is not the expected tag
    #[inline]
    pub const fn assert_tag(&self, tag: Tag) -> Result<()> {
        self.tag.assert_eq(tag)
    }

    /// Return error if object is not primitive (returning an `Error`)
    #[inline]
    pub const fn assert_primitive(&self) -> Result<()> {
        if self.is_primitive() {
            Ok(())
        } else {
            Err(Error::ConstructUnexpected)
        }
    }

    /// Return error if object is not primitive (returning an `InnerError`)
    #[inline]
    pub const fn assert_primitive_inner(&self) -> Result<(), InnerError> {
        if self.is_primitive() {
            Ok(())
        } else {
            Err(InnerError::ConstructUnexpected)
        }
    }

    /// Return error if object is not primitive (returning a `BerError`)
    #[inline]
    pub const fn assert_primitive_input<'i>(
        &'_ self,
        input: &'_ Input<'i>,
    ) -> Result<(), BerError<Input<'i>>> {
        if self.is_primitive() {
            Ok(())
        } else {
            Err(BerError::new(
                input.const_clone(),
                InnerError::ConstructUnexpected,
            ))
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

    /// Return error if object is primitive
    #[inline]
    pub const fn assert_constructed_inner(&self) -> Result<(), InnerError> {
        if !self.is_primitive() {
            Ok(())
        } else {
            Err(InnerError::ConstructExpected)
        }
    }

    /// Return error if object is primitive (returning a `BerError`)
    #[inline]
    pub const fn assert_constructed_input<'i>(
        &'_ self,
        input: &'_ Input<'i>,
    ) -> Result<(), BerError<Input<'i>>> {
        if !self.is_primitive() {
            Ok(())
        } else {
            Err(BerError::new(
                input.const_clone(),
                InnerError::ConstructExpected,
            ))
        }
    }

    /// Test if object class is Universal
    #[inline]
    pub const fn is_universal(&self) -> bool {
        self.class as u8 == Class::Universal as u8
    }
    /// Test if object class is Application
    #[inline]
    pub const fn is_application(&self) -> bool {
        self.class as u8 == Class::Application as u8
    }
    /// Test if object class is Context-specific
    #[inline]
    pub const fn is_contextspecific(&self) -> bool {
        self.class as u8 == Class::ContextSpecific as u8
    }
    /// Test if object class is Private
    #[inline]
    pub const fn is_private(&self) -> bool {
        self.class as u8 == Class::Private as u8
    }

    /// Return error if object length is definite
    #[inline]
    pub const fn assert_definite(&self) -> Result<()> {
        if self.length.is_definite() {
            Ok(())
        } else {
            Err(Error::DerConstraintFailed(DerConstraint::IndefiniteLength))
        }
    }

    /// Return error if object length is definite
    #[inline]
    pub const fn assert_definite_inner(&self) -> Result<(), InnerError> {
        if self.length.is_definite() {
            Ok(())
        } else {
            Err(InnerError::DerConstraintFailed(
                DerConstraint::IndefiniteLength,
            ))
        }
    }

    /// Get the content following a BER header
    #[inline]
    pub fn parse_ber_content<'i>(
        &'_ self,
        i: Input<'i>,
    ) -> IResult<Input<'i>, Input<'i>, BerError<Input<'i>>> {
        // defaults to maximum depth 8
        // depth is used only if BER, and length is indefinite
        BerMode::get_object_content(self, i, 8)
    }

    /// Get the content following a DER header
    #[inline]
    pub fn parse_der_content<'i>(
        &'_ self,
        i: Input<'i>,
    ) -> IResult<Input<'i>, Input<'i>, BerError<Input<'i>>> {
        self.assert_definite_inner()
            .map_err(BerError::convert(i.clone()))?;
        DerMode::get_object_content(self, i, 8)
    }
}

impl From<Tag> for Header<'_> {
    #[inline]
    fn from(tag: Tag) -> Self {
        let constructed = matches!(tag, Tag::Sequence | Tag::Set);
        Self::new(Class::Universal, constructed, tag, Length::Definite(0))
    }
}

impl ToStatic for Header<'_> {
    type Owned = Header<'static>;

    fn to_static(&self) -> Self::Owned {
        let raw_tag: Option<Cow<'static, [u8]>> =
            self.raw_tag.as_ref().map(|b| Cow::Owned(b.to_vec()));
        Header {
            tag: self.tag,
            constructed: self.constructed,
            class: self.class,
            length: self.length,
            raw_tag,
        }
    }
}

impl<'a> FromBer<'a> for Header<'a> {
    fn from_ber(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        parse_header(bytes).map_err(Err::convert)
    }
}

impl<'i> BerParser<'i> for Header<'i> {
    type Error = BerError<Input<'i>>;

    fn parse_ber(input: Input<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        parse_header(input)
    }

    /// <div class="warning">This method is usually not called (and will create a useless clone)</div>
    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        Ok((input, header.clone()))
    }
}

impl<'i> DerParser<'i> for Header<'i> {
    type Error = BerError<Input<'i>>;

    fn parse_der(input: Input<'i>) -> IResult<Input<'i>, Self, Self::Error> {
        let (rem, header) = parse_header(input.clone())?;
        // In DER, we reject Indefinite length
        if !header.length.is_definite() {
            return Err(Err::Error(BerError::new(
                input,
                InnerError::DerConstraintFailed(DerConstraint::IndefiniteLength),
            )));
        }
        Ok((rem, header))
    }

    /// <div class="warning">This method is usually not called (and will create a useless clone)</div>
    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        Ok((input, header.clone()))
    }
}

impl<'a> FromDer<'a> for Header<'a> {
    fn from_der(bytes: &'a [u8]) -> ParseResult<'a, Self> {
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
                return Err(Err::Error(Error::DerConstraintFailed(
                    DerConstraint::IndefiniteLength,
                )));
            }
            (_, l1) => {
                // if len is 0xff -> error (8.1.3.5)
                if l1 == 0b0111_1111 {
                    return Err(Err::Error(Error::InvalidLength));
                }
                // DER(9.1) if len is 0 (indefinite form), obj must be constructed
                der_constraint_fail_if!(
                    &i[1..],
                    len.1 == 0 && el.1 != 1,
                    DerConstraint::NotConstructed
                );
                let (i3, llen) = take(l1)(i2)?;
                match bytes_to_u64(llen) {
                    Ok(l) => {
                        // DER: should have been encoded in short form (< 127)
                        // XXX der_constraint_fail_if!(i, l < 127);
                        let l = usize::try_from(l).or(Err(Err::Error(Error::InvalidLength)))?;
                        (i3, Length::Definite(l))
                    }
                    Err(_) => {
                        return Err(Err::Error(Error::InvalidLength));
                    }
                }
            }
        };
        let constructed = el.1 != 0;
        let hdr = Header::new(class, constructed, Tag(el.2), len).with_raw_tag(Some(el.3.into()));
        Ok((i3, hdr))
    }
}

impl DynTagged for (Class, bool, Tag) {
    fn tag(&self) -> Tag {
        self.2
    }

    fn accept_tag(_: Tag) -> bool {
        true
    }
}

#[cfg(feature = "std")]
impl ToDer for (Class, bool, Tag) {
    fn to_der_len(&self) -> Result<usize> {
        let (_, _, tag) = self;
        match tag.0 {
            0..=30 => Ok(1),
            t => {
                let mut sz = 1;
                let mut val = t;
                loop {
                    if val <= 127 {
                        return Ok(sz + 1);
                    } else {
                        val >>= 7;
                        sz += 1;
                    }
                }
            }
        }
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let (class, constructed, tag) = self;
        let b0 = (*class as u8) << 6;
        let b0 = b0 | if *constructed { 0b10_0000 } else { 0 };
        if tag.0 > 30 {
            let mut val = tag.0;

            const BUF_SZ: usize = 8;
            let mut buffer = [0u8; BUF_SZ];
            let mut current_index = BUF_SZ - 1;

            // first byte: class+constructed+0x1f
            let b0 = b0 | 0b1_1111;
            let mut sz = writer.write(&[b0])?;

            // now write bytes from right (last) to left

            // last encoded byte
            buffer[current_index] = (val & 0x7f) as u8;
            val >>= 7;

            while val > 0 {
                current_index -= 1;
                if current_index == 0 {
                    return Err(SerializeError::InvalidLength);
                }
                buffer[current_index] = (val & 0x7f) as u8 | 0x80;
                val >>= 7;
            }

            sz += writer.write(&buffer[current_index..])?;
            Ok(sz)
        } else {
            let b0 = b0 | (tag.0 as u8);
            let sz = writer.write(&[b0])?;
            Ok(sz)
        }
    }

    fn write_der_content(&self, _writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        Ok(0)
    }
}

impl DynTagged for Header<'_> {
    fn tag(&self) -> Tag {
        self.tag
    }

    fn accept_tag(_: Tag) -> bool {
        true
    }
}

#[cfg(feature = "std")]
impl ToDer for Header<'_> {
    fn to_der_len(&self) -> Result<usize> {
        let tag_len = (self.class, self.constructed, self.tag).to_der_len()?;
        let len_len = self.length.to_der_len()?;
        Ok(tag_len + len_len)
    }

    fn write_der_header(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        let sz = (self.class, self.constructed, self.tag).write_der_header(writer)?;
        let sz = sz + self.length.write_der_header(writer)?;
        Ok(sz)
    }

    fn write_der_content(&self, _writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        Ok(0)
    }

    fn write_der_raw(&self, writer: &mut dyn std::io::Write) -> SerializeResult<usize> {
        // use raw_tag if present
        let sz = match &self.raw_tag {
            Some(t) => writer.write(t)?,
            None => (self.class, self.constructed, self.tag).write_der_header(writer)?,
        };
        let sz = sz + self.length.write_der_header(writer)?;
        Ok(sz)
    }
}

/// Compare two BER headers. `len` fields are compared only if both objects have it set (same for `raw_tag`)
impl<'a> PartialEq<Header<'a>> for Header<'a> {
    fn eq(&self, other: &Header) -> bool {
        self.class == other.class
            && self.tag == other.tag
            && self.constructed == other.constructed
            && {
                if self.length.is_null() && other.length.is_null() {
                    self.length == other.length
                } else {
                    true
                }
            }
            && {
                // it tag is present for both, compare it
                if self.raw_tag.as_ref().xor(other.raw_tag.as_ref()).is_none() {
                    self.raw_tag == other.raw_tag
                } else {
                    true
                }
            }
    }
}

impl Eq for Header<'_> {}

impl Hash for Header<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.class.hash(state);
        self.constructed.hash(state);
        self.tag.hash(state);
        self.length.hash(state);
        self.raw_tag.hash(state);
    }
}

pub(crate) fn parse_header<'a, I: nom::Input<Item = u8>>(
    input: I,
) -> IResult<I, Header<'a>, BerError<I>> {
    // parse identifier octets (X.690: 8.1.2)
    let (rem, b0) = be_u8(input.clone())?;

    // bits 8 and 7 represent the class of the tag
    let class_b0 = b0 >> 6;

    let class = match Class::try_from(class_b0) {
        Ok(c) => c,
        Err(_) => unreachable!(), // Cannot fail, we have read exactly 2 bits
    };

    const CONSTRUCTED_BIT: u8 = 0b0010_0000;
    // bit 6 shall be a 0 (primitive) or 1 (constructed)
    let constructed = (b0 & CONSTRUCTED_BIT) != 0;

    const TAG_MASK0: u8 = 0b0001_1111;
    // bits 5 to 1 encode the number of the tag
    let tag0 = b0 & TAG_MASK0;

    let mut rem = rem;
    let mut tag = u32::from(tag0);
    let mut tag_byte_count = 1;
    // test if tag >= 31 (X.690: 8.1.2.4)
    if tag0 == TAG_MASK0 {
        // read next bytes as specified in 8.1.2.4.2
        let mut c = 0;
        loop {
            // Make sure we don't read past the end of our data.
            let (r, b) = be_u8(rem).map_err(|_: Err<_, BerError<I>>| {
                Err::Error(BerError::new(input.clone(), InnerError::InvalidTag))
            })?;
            rem = r;

            // With tag defined as u32 the most we can fit in is four tag bytes.
            // (X.690 doesn't actually specify maximum tag width.)
            // custom_check!(i, tag_byte_count > 5, Error::InvalidTag)?;
            if tag_byte_count > 5 {
                return Err(Err::Error(BerError::new(input, InnerError::InvalidTag)));
            }

            c = (c << 7) | (u32::from(b) & 0x7f);
            let done = b & 0x80 == 0;
            tag_byte_count += 1;
            if done {
                break;
            }
        }
        tag = c;
    }

    // TODO: this code allocates & copies the raw tag. implement a borrowed version?
    let mut raw_tag = Vec::with_capacity(tag_byte_count);
    input.take(tag_byte_count).iter_elements().for_each(|item| {
        raw_tag.push(item);
    });

    // now parse length byte (X.690: 8.1.3)
    let (rem, len_b0) = be_u8(rem)?;
    let mut rem = rem;

    const INDEFINITE: u8 = 0b1000_0000;
    let length = if len_b0 == INDEFINITE {
        // indefinite form (X.690: 8.1.3.6)
        if !constructed {
            return Err(Err::Error(BerError::new(
                input,
                InnerError::IndefiniteLengthUnexpected,
            )));
        }
        Length::Indefinite
    } else if len_b0 & INDEFINITE == 0 {
        // definite, short form (X.690: 8.1.3.4)
        Length::Definite(len_b0 as usize)
    } else {
        // definite, long form (X.690: 8.1.3.5)

        // value 0b1111_1111 shall not be used (X.690: 8.1.3.5)
        if len_b0 == 0xff {
            return Err(Err::Error(BerError::new(input, InnerError::InvalidLength)));
        }
        let (r, len_bytes) = take(len_b0 & !INDEFINITE)(rem)?;
        rem = r;

        match bytes_to_u64_g(len_bytes) {
            Ok(l) => {
                let l = usize::try_from(l)
                    .map_err(|_| Err::Error(BerError::new(input, InnerError::InvalidLength)))?;
                Length::Definite(l)
            }
            _ => return Err(Err::Error(BerError::new(input, InnerError::InvalidLength))),
        }
    };

    let header =
        Header::new(class, constructed, Tag(tag), length).with_raw_tag(Some(Cow::Owned(raw_tag)));
    Ok((rem, header))
}

/// Try to parse *all* input bytes as u64
#[inline]
pub(crate) fn bytes_to_u64_g<I: nom::Input<Item = u8>>(s: I) -> Result<u64, InnerError> {
    let mut u: u64 = 0;
    for c in s.iter_elements() {
        if u & 0xff00_0000_0000_0000 != 0 {
            return Err(InnerError::IntegerTooLarge);
        }
        u <<= 8;
        u |= u64::from(c);
    }
    Ok(u)
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::*;
    use hex_literal::hex;

    /// Generic tests on methods, and coverage tests
    #[test]
    fn methods_header() {
        // Getters
        let input = &hex! {"02 01 00"};
        let (rem, header) = Header::from_ber(input).expect("parsing header failed");
        assert_eq!(header.class(), Class::Universal);
        assert_eq!(header.tag(), Tag::Integer);
        assert!(header.assert_primitive().is_ok());
        assert!(header.assert_constructed().is_err());
        assert!(header.is_universal());
        assert!(!header.is_application());
        assert!(!header.is_private());
        assert_eq!(rem, &input[2..]);

        // test PartialEq
        let hdr2 = Header::new_simple(Tag::Integer);
        assert_eq!(header, hdr2);

        // builder methods
        let hdr3 = hdr2
            .with_class(Class::ContextSpecific)
            .with_constructed(true)
            .with_length(Length::Definite(1));
        assert!(hdr3.constructed());
        assert!(hdr3.is_constructed());
        assert!(hdr3.assert_constructed().is_ok());
        assert!(hdr3.is_contextspecific());
        let xx = hdr3.to_der_vec().expect("serialize failed");
        assert_eq!(&xx, &[0xa2, 0x01]);

        // indefinite length
        let hdr4 = hdr3.with_length(Length::Indefinite);
        assert!(hdr4.assert_definite().is_err());
        let xx = hdr4.to_der_vec().expect("serialize failed");
        assert_eq!(&xx, &[0xa2, 0x80]);

        // indefinite length should be accepted only if constructed
        let primitive_indef = &hex!("0280");
        Header::parse_ber(primitive_indef.into()).expect_err("primitive with indefinite length");

        // parse_*_content
        let hdr = Header::new_simple(Tag(2)).with_length(Length::Definite(1));
        let i: Input = (&input[2..]).into();
        let (_, r) = hdr.parse_ber_content(i.clone()).unwrap();
        assert_eq!(r.as_bytes2(), &input[2..]);
        let (_, r) = hdr.parse_der_content(i).unwrap();
        assert_eq!(r.as_bytes2(), &input[2..]);
    }
}
