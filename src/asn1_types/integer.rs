use crate::debug::trace_input;
use crate::*;
use alloc::borrow::Cow;
use alloc::vec;
use nom::Input as _;

#[cfg(feature = "bigint")]
#[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
pub use num_bigint::{BigInt, BigUint, Sign};

/// Decode an unsigned integer into a big endian byte slice with all leading
/// zeroes removed (if positive) and extra 0xff removed (if negative)
fn trim_slice(bytes: &[u8]) -> &[u8] {
    if bytes.is_empty() || (bytes[0] != 0x00 && bytes[0] != 0xff) {
        return bytes;
    }

    match bytes.iter().position(|&b| b != 0) {
        // first byte is not 0
        Some(0) => (),
        // all bytes are 0
        None => return &bytes[bytes.len() - 1..],
        Some(first) => return &bytes[first..],
    }

    // same for negative integers : skip byte 0->n if byte 0->n = 0xff AND byte n+1 >= 0x80
    match bytes.windows(2).position(|s| match s {
        &[a, b] => !(a == 0xff && b >= 0x80),
        _ => true,
    }) {
        // first byte is not 0xff
        Some(0) => (),
        // all bytes are 0xff
        None => return &bytes[bytes.len() - 1..],
        Some(first) => return &bytes[first..],
    }

    bytes
}

/// Decode an unsigned integer into a byte array of the requested size
/// containing a big endian integer.
fn decode_array_uint<const N: usize>(bytes: &[u8]) -> Result<[u8; N]> {
    if is_highest_bit_set(bytes) {
        return Err(Error::IntegerNegative);
    }
    let input = trim_slice(bytes);

    if input.len() > N {
        return Err(Error::IntegerTooLarge);
    }

    // Input has leading zeroes removed, so we need to add them back
    let mut output = [0u8; N];
    assert!(input.len() <= N);
    output[N.saturating_sub(input.len())..].copy_from_slice(input);
    Ok(output)
}

/// Decode an unsigned integer of the specified size.
///
/// Returns a byte array of the requested size containing a big endian integer.
fn decode_array_int<const N: usize>(bytes: &[u8]) -> Result<[u8; N]> {
    if bytes.len() > N {
        return Err(Error::IntegerTooLarge);
    }

    // any.tag().assert_eq(Tag::Integer)?;
    let mut output = [0xFFu8; N];
    let offset = N.saturating_sub(bytes.len());
    output[offset..].copy_from_slice(bytes);
    Ok(output)
}

/// Is the highest bit of the first byte in the slice 1? (if present)
#[inline]
fn is_highest_bit_set(bytes: &[u8]) -> bool {
    bytes
        .first()
        .map(|byte| byte & 0b10000000 != 0)
        .unwrap_or(false)
}

macro_rules! impl_int {
    ($uint:ty => $int:ty) => {
        impl_tryfrom_any!($int);

        impl<'i> BerParser<'i> for $int {
            type Error = BerError<Input<'i>>;

            fn from_ber_content(
                header: &'_ Header<'i>,
                input: Input<'i>,
            ) -> IResult<Input<'i>, Self, Self::Error> {
                // Encoding shall be primitive (X.690: 8.3.1)
                header.assert_primitive_input(&input).map_err(Err::Error)?;

                let uint = if is_highest_bit_set(input.as_bytes2()) {
                    let ar = decode_array_int(input.as_bytes2())
                        .map_err(|e| BerError::nom_err_input(&input, e.into()))?;
                    <$uint>::from_be_bytes(ar)
                } else {
                    // read as uint, but check if the value will fit in a signed integer
                    let ar = decode_array_uint(input.as_bytes2())
                        .map_err(|e| BerError::nom_err_input(&input, e.into()))?;
                    let u = <$uint>::from_be_bytes(ar);
                    if u > <$int>::MAX as $uint {
                        return Err(BerError::nom_err_input(&input, InnerError::IntegerTooLarge));
                    }
                    u
                };
                // decode_array_* consume all bytes, so return empty input with result
                Ok((input.take_from(input.len()), uint as $int))
            }
        }

        impl<'i> DerParser<'i> for $int {
            type Error = BerError<Input<'i>>;

            fn from_der_content(
                header: &'_ Header<'i>,
                input: Input<'i>,
            ) -> IResult<Input<'i>, Self, Self::Error> {
                // Note: should we relax this constraint (leading 00 or ff)?
                check_der_int_constraints_input(&input).map_err(|e| {
                    BerError::nom_err_input(&input, InnerError::DerConstraintFailed(e))
                })?;

                Self::from_ber_content(header, input)
            }
        }

        impl CheckDerConstraints for $int {
            fn check_constraints(any: &Any) -> Result<()> {
                check_der_int_constraints(any)
            }
        }

        impl DerAutoDerive for $int {}

        impl Tagged for $int {
            const TAG: Tag = Tag::Integer;
        }

        #[cfg(feature = "std")]
        const _: () = {
            use std::io::Write;

            impl ToBer for $int {
                type Encoder = Primitive<{ Tag::Integer.0 }>;

                fn ber_content_len(&self) -> Length {
                    let int = Integer::from(*self);
                    Length::Definite(int.data.len())
                }

                fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
                    let int = Integer::from(*self);
                    target.write(&int.data).map_err(Into::into)
                }

                fn ber_tag_info(&self) -> (Class, bool, Tag) {
                    use $crate::Tagged;
                    (Self::CLASS, false, Self::TAG)
                }
            }

            impl ToDer for $int {
                type Encoder = Primitive<{ Tag::Integer.0 }>;

                fn der_content_len(&self) -> Length {
                    let int = Integer::from(*self);
                    Length::Definite(int.data.len())
                }

                fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
                    let int = Integer::from(*self);
                    target.write(&int.data).map_err(Into::into)
                }

                fn der_tag_info(&self) -> (Class, bool, Tag) {
                    use $crate::Tagged;
                    (Self::CLASS, false, Self::TAG)
                }
            }
        };
    };
}

macro_rules! impl_uint {
    ($ty:ty) => {
        impl_tryfrom_any!($ty);

        impl<'i> BerParser<'i> for $ty {
            type Error = BerError<Input<'i>>;

            fn from_ber_content(
                header: &'_ Header<'i>,
                input: Input<'i>,
            ) -> IResult<Input<'i>, Self, Self::Error> {
                // Encoding shall be primitive (X.690: 8.3.1)
                header.assert_primitive_input(&input).map_err(Err::Error)?;

                let ar = decode_array_uint(input.as_bytes2())
                    .map_err(|e| BerError::nom_err_input(&input, e.into()))?;
                let uint = Self::from_be_bytes(ar);

                // decode_array_* consume all bytes, so return empty input with result
                Ok((input.take_from(input.len()), uint))
            }
        }

        impl<'i> DerParser<'i> for $ty {
            type Error = BerError<Input<'i>>;

            fn from_der_content(
                header: &'_ Header<'i>,
                input: Input<'i>,
            ) -> IResult<Input<'i>, Self, Self::Error> {
                // Note: should we relax this constraint (leading 00 or ff)?
                check_der_int_constraints_input(&input).map_err(|e| {
                    BerError::nom_err_input(&input, InnerError::DerConstraintFailed(e))
                })?;

                Self::from_ber_content(header, input)
            }
        }

        impl CheckDerConstraints for $ty {
            fn check_constraints(any: &Any) -> Result<()> {
                check_der_int_constraints(any)
            }
        }

        impl DerAutoDerive for $ty {}

        impl Tagged for $ty {
            const TAG: Tag = Tag::Integer;
        }

        #[cfg(feature = "std")]
        const _: () = {
            use std::io::Write;

            impl ToBer for $ty {
                type Encoder = Primitive<{ Tag::Integer.0 }>;

                fn ber_content_len(&self) -> Length {
                    let int = Integer::from(*self);
                    Length::Definite(int.data.len())
                }

                fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
                    let int = Integer::from(*self);
                    target.write(&int.data).map_err(Into::into)
                }

                fn ber_tag_info(&self) -> (Class, bool, Tag) {
                    use $crate::Tagged;
                    (Self::CLASS, false, Self::TAG)
                }
            }

            impl ToDer for $ty {
                type Encoder = Primitive<{ Tag::Integer.0 }>;

                fn der_content_len(&self) -> Length {
                    let int = Integer::from(*self);
                    Length::Definite(int.data.len())
                }

                fn der_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
                    let int = Integer::from(*self);
                    target.write(&int.data).map_err(Into::into)
                }

                fn der_tag_info(&self) -> (Class, bool, Tag) {
                    use $crate::Tagged;
                    (Self::CLASS, false, Self::TAG)
                }
            }
        };
    };
}

impl_uint!(u8);
impl_uint!(u16);
impl_uint!(u32);
impl_uint!(u64);
impl_uint!(u128);
impl_uint!(usize);
impl_int!(u8 => i8);
impl_int!(u16 => i16);
impl_int!(u32 => i32);
impl_int!(u64 => i64);
impl_int!(u128 => i128);
impl_int!(usize => isize);

/// ASN.1 `INTEGER` type
///
/// Generic representation for integer types.
/// BER/DER integers can be of any size, so it is not possible to store them as simple integers (they
/// are stored as raw bytes).
///
/// The internal representation can be obtained using `.as_ref()`.
///
/// # Note
///
/// Methods from/to BER and DER encodings are also implemented for primitive types
/// (`u8`, `u16` to `u128`, and `i8` to `i128`).
/// In most cases, it is easier to use these types directly.
///
/// # Examples
///
/// Creating an `Integer`
///
/// ```
/// use asn1_rs::Integer;
///
/// // unsigned
/// let i = Integer::from(4);
/// assert_eq!(i.as_ref(), &[4]);
/// // signed
/// let j = Integer::from(-2);
/// assert_eq!(j.as_ref(), &[0xfe]);
/// ```
///
/// Converting an `Integer` to a primitive type (using the `TryInto` trait)
///
/// ```
/// use asn1_rs::{Error, Integer};
/// use std::convert::TryInto;
///
/// let i = Integer::new(&[0x12, 0x34, 0x56, 0x78]);
/// // converts to an u32
/// let n: u32 = i.try_into().unwrap();
///
/// // Same, but converting to an u16: will fail, value cannot fit into an u16
/// let i = Integer::new(&[0x12, 0x34, 0x56, 0x78]);
/// assert_eq!(i.try_into() as Result<u16, _>, Err(Error::IntegerTooLarge));
/// ```
///
/// Encoding an `Integer` to DER
///
#[cfg_attr(feature = "std", doc = r#"```"#)]
#[cfg_attr(not(feature = "std"), doc = r#"```rust,compile_fail"#)]
/// use asn1_rs::{Integer, ToDer};
///
/// let i = Integer::from(4);
/// let v = i.to_der_vec().unwrap();
/// assert_eq!(&v, &[2, 1, 4]);
///
/// // same, with primitive types
/// let v = 4.to_der_vec().unwrap();
/// assert_eq!(&v, &[2, 1, 4]);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Integer<'a> {
    pub(crate) data: Cow<'a, [u8]>,
}

impl<'a> Integer<'a> {
    /// Creates a new `Integer` containing the given value (borrowed).
    #[inline]
    pub const fn new(s: &'a [u8]) -> Self {
        Integer {
            data: Cow::Borrowed(s),
        }
    }

    /// Creates a borrowed `Any` for this object
    #[inline]
    pub fn any(&'a self) -> Any<'a> {
        Any::from_tag_and_data(Self::TAG, self.data.as_ref().into())
    }

    /// Return a reference to the raw data, if shared
    ///
    /// Note: unlike `.as_ref()`, this function can return a reference that can
    /// outlive the current object (if the raw data does).
    #[inline]
    pub fn as_raw_slice(&self) -> Option<&'a [u8]> {
        match self.data {
            Cow::Borrowed(s) => Some(s),
            _ => None,
        }
    }

    /// Returns a `BigInt` built from this `Integer` value.
    #[cfg(feature = "bigint")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
    pub fn as_bigint(&self) -> BigInt {
        BigInt::from_signed_bytes_be(&self.data)
    }

    /// Returns a `BigUint` built from this `Integer` value.
    #[cfg(feature = "bigint")]
    #[cfg_attr(docsrs, doc(cfg(feature = "bigint")))]
    pub fn as_biguint(&self) -> Result<BigUint> {
        if is_highest_bit_set(&self.data) {
            Err(Error::IntegerNegative)
        } else {
            Ok(BigUint::from_bytes_be(&self.data))
        }
    }

    /// Build an `Integer` from a constant array of bytes representation of an integer.
    pub fn from_const_array<const N: usize>(b: [u8; N]) -> Self {
        // if high bit set -> add leading 0 to ensure unsigned
        if is_highest_bit_set(&b) {
            let mut bytes = vec![0];
            bytes.extend_from_slice(&b);

            Integer {
                data: Cow::Owned(bytes),
            }
        }
        // otherwise -> remove 0 unless next has high bit set
        else {
            let mut idx = 0;

            while idx < b.len() - 1 {
                if b[idx] == 0 && b[idx + 1] < 0x80 {
                    idx += 1;
                    continue;
                }
                break;
            }

            Integer {
                data: Cow::Owned(b[idx..].to_vec()),
            }
        }
    }

    fn from_const_array_negative<const N: usize>(b: [u8; N]) -> Self {
        let mut idx = 0;

        // Skip leading FF unless next has high bit clear
        while idx < b.len() - 1 {
            if b[idx] == 0xFF && b[idx + 1] >= 0x80 {
                idx += 1;
                continue;
            }
            break;
        }

        if idx == b.len() {
            Integer {
                data: Cow::Borrowed(&[0]),
            }
        } else {
            Integer {
                data: Cow::Owned(b[idx..].to_vec()),
            }
        }
    }
}

macro_rules! impl_from_to {
    ($ty:ty, $sty:expr, $from:ident, $to:ident) => {
        impl From<$ty> for Integer<'_> {
            fn from(i: $ty) -> Self {
                Self::$from(i)
            }
        }

        impl core::convert::TryFrom<Integer<'_>> for $ty {
            type Error = Error;

            fn try_from(value: Integer<'_>) -> Result<Self> {
                value.$to()
            }
        }

        impl Integer<'_> {
            #[doc = "Attempts to convert an `Integer` to a `"]
            #[doc = $sty]
            #[doc = "`."]
            #[doc = ""]
            #[doc = "This function returns an `IntegerTooLarge` error if the integer will not fit into the output type."]
            pub fn $to(&self) -> Result<$ty> {
                use core::convert::TryInto;
                self.any().try_into()
            }
        }
    };
    (IMPL SIGNED $ty:ty, $sty:expr, $from:ident, $to:ident) => {
        impl_from_to!($ty, $sty, $from, $to);

        impl Integer<'_> {
            #[doc = "Converts a `"]
            #[doc = $sty]
            #[doc = "` to an `Integer`"]
            #[doc = ""]
            #[doc = "Note: this function allocates data."]
            pub fn $from(i: $ty) -> Self {
                let b = i.to_be_bytes();
                if i >= 0 {
                    Self::from_const_array(b)
                } else {
                    Self::from_const_array_negative(b)
                }
            }
        }
    };
    (IMPL UNSIGNED $ty:ty, $sty:expr, $from:ident, $to:ident) => {
        impl_from_to!($ty, $sty, $from, $to);

        impl Integer<'_> {
            #[doc = "Converts a `"]
            #[doc = $sty]
            #[doc = "` to an `Integer`"]
            #[doc = ""]
            #[doc = "Note: this function allocates data."]
            pub fn $from(i: $ty) -> Self {
                Self::from_const_array(i.to_be_bytes())
            }
        }
    };
    (SIGNED $ty:ty, $from:ident, $to:ident) => {
        impl_from_to!(IMPL SIGNED $ty, stringify!($ty), $from, $to);
    };
    (UNSIGNED $ty:ty, $from:ident, $to:ident) => {
        impl_from_to!(IMPL UNSIGNED $ty, stringify!($ty), $from, $to);
    };
}

impl_from_to!(SIGNED i8, from_i8, as_i8);
impl_from_to!(SIGNED i16, from_i16, as_i16);
impl_from_to!(SIGNED i32, from_i32, as_i32);
impl_from_to!(SIGNED i64, from_i64, as_i64);
impl_from_to!(SIGNED i128, from_i128, as_i128);
impl_from_to!(SIGNED isize, from_isize, as_isize);

impl_from_to!(UNSIGNED u8, from_u8, as_u8);
impl_from_to!(UNSIGNED u16, from_u16, as_u16);
impl_from_to!(UNSIGNED u32, from_u32, as_u32);
impl_from_to!(UNSIGNED u64, from_u64, as_u64);
impl_from_to!(UNSIGNED u128, from_u128, as_u128);
impl_from_to!(UNSIGNED usize, from_usize, as_usize);

impl AsRef<[u8]> for Integer<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl_tryfrom_any!('i @ Integer<'i>);

impl<'i> BerParser<'i> for Integer<'i> {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        trace_input("Integer::from_ber_content", |input| {
            // Encoding shall be primitive (X.690: 8.3.1)
            header.assert_primitive_input(&input).map_err(Err::Error)?;

            // since encoding must be primitive, indefinite length is not allowed
            // so we use `der_get_content`
            let (rem, content) = der_get_content(header, input)?;

            // The contents octets shall consist of one or more octets (X.690: 8.3.2)
            if content.is_empty() {
                return Err(BerError::nom_err(content, InnerError::InvalidLength));
            }

            Ok((
                rem,
                Integer {
                    data: Cow::Borrowed(content.as_bytes2()),
                },
            ))
        })(input)
    }
}

impl<'i> DerParser<'i> for Integer<'i> {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Note: should we relax this constraint (leading 00 or ff)?
        check_der_int_constraints_input(&input)
            .map_err(|e| BerError::nom_err_input(&input, InnerError::DerConstraintFailed(e)))?;

        Self::from_ber_content(header, input)
    }
}

impl CheckDerConstraints for Integer<'_> {
    fn check_constraints(any: &Any) -> Result<()> {
        check_der_int_constraints(any)
    }
}

fn check_der_int_constraints(any: &Any) -> Result<()> {
    any.header.assert_primitive()?;
    any.header.length.assert_definite()?;
    match any.as_bytes() {
        [] => Err(Error::DerConstraintFailed(DerConstraint::IntegerEmpty)),
        [0] => Ok(()),
        // leading zeroes
        [0, byte, ..] if *byte < 0x80 => Err(Error::DerConstraintFailed(
            DerConstraint::IntegerLeadingZeroes,
        )),
        // negative integer with non-minimal encoding
        [0xff, byte, ..] if *byte >= 0x80 => {
            Err(Error::DerConstraintFailed(DerConstraint::IntegerLeadingFF))
        }
        _ => Ok(()),
    }
}

fn check_der_int_constraints_input(input: &Input) -> Result<(), DerConstraint> {
    match input.as_bytes2() {
        [] => Err(DerConstraint::IntegerEmpty),
        [0] => Ok(()),
        // leading zeroes
        [0, byte, ..] if *byte < 0x80 => Err(DerConstraint::IntegerLeadingZeroes),
        // negative integer with non-minimal encoding
        [0xff, byte, ..] if *byte >= 0x80 => Err(DerConstraint::IntegerLeadingFF),
        _ => Ok(()),
    }
}

impl DerAutoDerive for Integer<'_> {}

impl Tagged for Integer<'_> {
    const TAG: Tag = Tag::Integer;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for Integer<'_> {
        type Encoder = Primitive<{ Tag::Integer.0 }>;

        fn ber_content_len(&self) -> Length {
            Length::Definite(self.data.len())
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write(&self.data).map_err(Into::into)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(LFT 'a, Integer<'a>);
};

/// Helper macro to declare integers at compile-time
///
/// [`Integer`] stores the encoded representation of the integer, so declaring
/// an integer requires to either use a runtime function or provide the encoded value.
/// This macro simplifies this task by encoding the value.
/// It can be used the following ways:
///
/// - `int!(1234)`: Create a const expression for the corresponding `Integer<'static>`
/// - `int!(raw 1234)`: Return the DER encoded form as a byte array (hex-encoded, big-endian
///   representation from the integer, with leading zeroes removed).
///
/// # Examples
///
/// ```rust
/// use asn1_rs::{int, Integer};
///
/// const INT0: Integer = int!(1234);
/// ```
#[macro_export]
macro_rules! int {
    (raw $item:expr) => {
        $crate::exports::asn1_rs_impl::encode_int!($item)
    };
    (rel $item:expr) => {
        $crate::exports::asn1_rs_impl::encode_int!(rel $item)
    };
    ($item:expr) => {
        $crate::Integer::new(
            &$crate::int!(raw $item),
        )
    };
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::{BerParser, DerParser, FromDer, Input, ToDer};
    use std::convert::TryInto;

    // Vectors from Section 5.7 of:
    // https://luca.ntop.org/Teaching/Appunti/asn1.html
    pub(crate) const I0_BYTES: &[u8] = &[0x02, 0x01, 0x00];
    pub(crate) const I127_BYTES: &[u8] = &[0x02, 0x01, 0x7F];
    pub(crate) const I128_BYTES: &[u8] = &[0x02, 0x02, 0x00, 0x80];
    pub(crate) const I256_BYTES: &[u8] = &[0x02, 0x02, 0x01, 0x00];
    pub(crate) const INEG128_BYTES: &[u8] = &[0x02, 0x01, 0x80];
    pub(crate) const INEG129_BYTES: &[u8] = &[0x02, 0x02, 0xFF, 0x7F];

    // Additional vectors
    pub(crate) const I255_BYTES: &[u8] = &[0x02, 0x02, 0x00, 0xFF];
    pub(crate) const I32767_BYTES: &[u8] = &[0x02, 0x02, 0x7F, 0xFF];
    pub(crate) const I65535_BYTES: &[u8] = &[0x02, 0x03, 0x00, 0xFF, 0xFF];
    pub(crate) const INEG32768_BYTES: &[u8] = &[0x02, 0x02, 0x80, 0x00];

    #[test]
    fn decode_i8() {
        assert_eq!(0, i8::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, i8::from_der(I127_BYTES).unwrap().1);
        assert_eq!(-128, i8::from_der(INEG128_BYTES).unwrap().1);

        type T = i8;
        const TEST_VECTORS: &[(T, &[u8])] =
            &[(0, I0_BYTES), (127, I127_BYTES), (-128, INEG128_BYTES)];

        for (expected, bytes) in TEST_VECTORS {
            assert_eq!(*expected, <T>::parse_ber((*bytes).into()).unwrap().1);
            assert_eq!(*expected, <T>::parse_der((*bytes).into()).unwrap().1);
        }
    }

    #[test]
    fn encode_i8() {
        assert_eq!(0i8.to_der_vec().unwrap(), I0_BYTES);
        assert_eq!(127i8.to_der_vec().unwrap(), I127_BYTES);
        assert_eq!((-128i8).to_der_vec().unwrap(), INEG128_BYTES);
    }

    #[test]
    fn decode_i16() {
        assert_eq!(0, i16::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, i16::from_der(I127_BYTES).unwrap().1);
        assert_eq!(128, i16::from_der(I128_BYTES).unwrap().1);
        assert_eq!(255, i16::from_der(I255_BYTES).unwrap().1);
        assert_eq!(256, i16::from_der(I256_BYTES).unwrap().1);
        assert_eq!(32767, i16::from_der(I32767_BYTES).unwrap().1);
        assert_eq!(-128, i16::from_der(INEG128_BYTES).unwrap().1);
        assert_eq!(-129, i16::from_der(INEG129_BYTES).unwrap().1);
        assert_eq!(-32768, i16::from_der(INEG32768_BYTES).unwrap().1);

        type T = i16;
        const TEST_VECTORS: &[(T, &[u8])] = &[
            (0, I0_BYTES),
            (127, I127_BYTES),
            (128, I128_BYTES),
            (255, I255_BYTES),
            (256, I256_BYTES),
            (32767, I32767_BYTES),
            (-128, INEG128_BYTES),
            (-129, INEG129_BYTES),
            (-32768, INEG32768_BYTES),
        ];

        for (expected, bytes) in TEST_VECTORS {
            assert_eq!(*expected, <T>::parse_ber((*bytes).into()).unwrap().1);
            assert_eq!(*expected, <T>::parse_der((*bytes).into()).unwrap().1);
        }
    }

    #[test]
    fn encode_i16() {
        assert_eq!(0i16.to_der_vec().unwrap(), I0_BYTES);
        assert_eq!(127i16.to_der_vec().unwrap(), I127_BYTES);
        assert_eq!(128i16.to_der_vec().unwrap(), I128_BYTES);
        assert_eq!(255i16.to_der_vec().unwrap(), I255_BYTES);
        assert_eq!(256i16.to_der_vec().unwrap(), I256_BYTES);
        assert_eq!(32767i16.to_der_vec().unwrap(), I32767_BYTES);
        assert_eq!((-128i16).to_der_vec().unwrap(), INEG128_BYTES);
        assert_eq!((-129i16).to_der_vec().unwrap(), INEG129_BYTES);
        assert_eq!((-32768i16).to_der_vec().unwrap(), INEG32768_BYTES);
    }

    #[test]
    fn decode_isize() {
        assert_eq!(0, isize::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, isize::from_der(I127_BYTES).unwrap().1);
        assert_eq!(128, isize::from_der(I128_BYTES).unwrap().1);
        assert_eq!(255, isize::from_der(I255_BYTES).unwrap().1);
        assert_eq!(256, isize::from_der(I256_BYTES).unwrap().1);
        assert_eq!(32767, isize::from_der(I32767_BYTES).unwrap().1);
        assert_eq!(-128, isize::from_der(INEG128_BYTES).unwrap().1);
        assert_eq!(-129, isize::from_der(INEG129_BYTES).unwrap().1);
        assert_eq!(-32768, isize::from_der(INEG32768_BYTES).unwrap().1);

        type T = isize;
        const TEST_VECTORS: &[(T, &[u8])] = &[
            (0, I0_BYTES),
            (127, I127_BYTES),
            (128, I128_BYTES),
            (255, I255_BYTES),
            (256, I256_BYTES),
            (32767, I32767_BYTES),
            (-128, INEG128_BYTES),
            (-129, INEG129_BYTES),
            (-32768, INEG32768_BYTES),
        ];

        for (expected, bytes) in TEST_VECTORS {
            assert_eq!(*expected, <T>::parse_ber((*bytes).into()).unwrap().1);
            assert_eq!(*expected, <T>::parse_der((*bytes).into()).unwrap().1);
        }
    }

    #[test]
    fn encode_isize() {
        assert_eq!(0isize.to_der_vec().unwrap(), I0_BYTES);
        assert_eq!(127isize.to_der_vec().unwrap(), I127_BYTES);
        assert_eq!(128isize.to_der_vec().unwrap(), I128_BYTES);
        assert_eq!(255isize.to_der_vec().unwrap(), I255_BYTES);
        assert_eq!(256isize.to_der_vec().unwrap(), I256_BYTES);
        assert_eq!(32767isize.to_der_vec().unwrap(), I32767_BYTES);
        assert_eq!((-128isize).to_der_vec().unwrap(), INEG128_BYTES);
        assert_eq!((-129isize).to_der_vec().unwrap(), INEG129_BYTES);
        assert_eq!((-32768isize).to_der_vec().unwrap(), INEG32768_BYTES);
    }

    #[test]
    fn decode_u8() {
        assert_eq!(0, u8::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, u8::from_der(I127_BYTES).unwrap().1);
        assert_eq!(255, u8::from_der(I255_BYTES).unwrap().1);

        type T = u8;
        const TEST_VECTORS: &[(T, &[u8])] = &[
            (0, I0_BYTES),
            (127, I127_BYTES),
            (128, I128_BYTES),
            (255, I255_BYTES),
        ];

        for (expected, bytes) in TEST_VECTORS {
            assert_eq!(*expected, <T>::parse_ber((*bytes).into()).unwrap().1);
            assert_eq!(*expected, <T>::parse_der((*bytes).into()).unwrap().1);
        }
    }

    #[test]
    fn encode_u8() {
        assert_eq!(0u8.to_der_vec().unwrap(), I0_BYTES);
        assert_eq!(127u8.to_der_vec().unwrap(), I127_BYTES);
        assert_eq!(255u8.to_der_vec().unwrap(), I255_BYTES);
    }

    #[test]
    fn decode_u16() {
        assert_eq!(0, u16::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, u16::from_der(I127_BYTES).unwrap().1);
        assert_eq!(255, u16::from_der(I255_BYTES).unwrap().1);
        assert_eq!(256, u16::from_der(I256_BYTES).unwrap().1);
        assert_eq!(32767, u16::from_der(I32767_BYTES).unwrap().1);
        assert_eq!(65535, u16::from_der(I65535_BYTES).unwrap().1);

        type T = u16;
        const TEST_VECTORS: &[(T, &[u8])] = &[
            (0, I0_BYTES),
            (127, I127_BYTES),
            (128, I128_BYTES),
            (255, I255_BYTES),
            (256, I256_BYTES),
            (32767, I32767_BYTES),
            (65535, I65535_BYTES),
        ];

        for (expected, bytes) in TEST_VECTORS {
            assert_eq!(*expected, <T>::parse_ber((*bytes).into()).unwrap().1);
            assert_eq!(*expected, <T>::parse_der((*bytes).into()).unwrap().1);
        }
    }

    #[test]
    fn encode_u16() {
        assert_eq!(0u16.to_der_vec().unwrap(), I0_BYTES);
        assert_eq!(127u16.to_der_vec().unwrap(), I127_BYTES);
        assert_eq!(255u16.to_der_vec().unwrap(), I255_BYTES);
        assert_eq!(256u16.to_der_vec().unwrap(), I256_BYTES);
        assert_eq!(32767u16.to_der_vec().unwrap(), I32767_BYTES);
        assert_eq!(65535u16.to_der_vec().unwrap(), I65535_BYTES);
    }

    #[test]
    fn decode_usize() {
        assert_eq!(0, usize::from_der(I0_BYTES).unwrap().1);
        assert_eq!(127, usize::from_der(I127_BYTES).unwrap().1);
        assert_eq!(255, usize::from_der(I255_BYTES).unwrap().1);
        assert_eq!(256, usize::from_der(I256_BYTES).unwrap().1);
        assert_eq!(32767, usize::from_der(I32767_BYTES).unwrap().1);
        assert_eq!(65535, usize::from_der(I65535_BYTES).unwrap().1);

        type T = usize;
        const TEST_VECTORS: &[(T, &[u8])] = &[
            (0, I0_BYTES),
            (127, I127_BYTES),
            (128, I128_BYTES),
            (255, I255_BYTES),
            (256, I256_BYTES),
            (32767, I32767_BYTES),
            (65535, I65535_BYTES),
        ];

        for (expected, bytes) in TEST_VECTORS {
            assert_eq!(*expected, <T>::parse_ber((*bytes).into()).unwrap().1);
            assert_eq!(*expected, <T>::parse_der((*bytes).into()).unwrap().1);
        }
    }

    #[test]
    fn encode_usize() {
        assert_eq!(0usize.to_der_vec().unwrap(), I0_BYTES);
        assert_eq!(127usize.to_der_vec().unwrap(), I127_BYTES);
        assert_eq!(255usize.to_der_vec().unwrap(), I255_BYTES);
        assert_eq!(256usize.to_der_vec().unwrap(), I256_BYTES);
        assert_eq!(32767usize.to_der_vec().unwrap(), I32767_BYTES);
        assert_eq!(65535usize.to_der_vec().unwrap(), I65535_BYTES);
    }

    /// Integers must be encoded with a minimum number of octets
    #[test]
    fn reject_non_canonical() {
        assert!(i8::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(i16::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(u8::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());
        assert!(u16::from_der(&[0x02, 0x02, 0x00, 0x00]).is_err());

        assert!(i8::parse_der(Input::from(&[0x02, 0x02, 0x00, 0x00])).is_err());
        assert!(i16::parse_der(Input::from(&[0x02, 0x02, 0x00, 0x00])).is_err());
        assert!(i8::parse_der(Input::from(&[0x02, 0x02, 0xff, 0xff])).is_err());
        assert!(i16::parse_der(Input::from(&[0x02, 0x02, 0xff, 0xff])).is_err());

        assert!(u8::parse_der(Input::from(&[0x02, 0x02, 0x00, 0x00])).is_err());
        assert!(u16::parse_der(Input::from(&[0x02, 0x02, 0x00, 0x00])).is_err());
    }

    #[test]
    fn declare_int() {
        let int = super::int!(1234);
        assert_eq!(int.try_into(), Ok(1234));
    }

    #[test]
    fn trim_slice() {
        use super::trim_slice;
        // no zero nor ff - nothing to remove
        let input: &[u8] = &[0x7f, 0xff, 0x00, 0x02];
        assert_eq!(input, trim_slice(input));
        //
        // 0x00
        //
        // empty - nothing to remove
        let input: &[u8] = &[];
        assert_eq!(input, trim_slice(input));
        // one zero - nothing to remove
        let input: &[u8] = &[0];
        assert_eq!(input, trim_slice(input));
        // all zeroes - keep only one
        let input: &[u8] = &[0, 0, 0];
        assert_eq!(&input[2..], trim_slice(input));
        // some zeroes - keep only the non-zero part
        let input: &[u8] = &[0, 0, 1];
        assert_eq!(&input[2..], trim_slice(input));
        //
        // 0xff
        //
        // one ff - nothing to remove
        let input: &[u8] = &[0xff];
        assert_eq!(input, trim_slice(input));
        // all ff - keep only one
        let input: &[u8] = &[0xff, 0xff, 0xff];
        assert_eq!(&input[2..], trim_slice(input));
        // some ff - keep only the non-zero part
        let input: &[u8] = &[0xff, 0xff, 1];
        assert_eq!(&input[1..], trim_slice(input));
        // some ff and a MSB 1 - keep only the non-zero part
        let input: &[u8] = &[0xff, 0xff, 0x80, 1];
        assert_eq!(&input[2..], trim_slice(input));
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{Integer, ToBer};

        #[test]
        fn tober_integer() {
            // Ok: Integer
            let i = Integer::from(4);
            let mut v: Vec<u8> = Vec::new();
            i.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"020104"});

            // Ok: integer with MSB set (must be encoded on 2 bytes)
            let i = Integer::from(0x8f);
            let mut v: Vec<u8> = Vec::new();
            i.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"0202008f"});

            //---- u8
            let i = 4_u8;
            let mut v: Vec<u8> = Vec::new();
            i.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"020104"});

            let i = 0x8f_u8;
            let mut v: Vec<u8> = Vec::new();
            i.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"0202008f"});

            //---- i8
            let i = 4_i8;
            let mut v: Vec<u8> = Vec::new();
            i.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"020104"});

            let i = -4_i8;
            let mut v: Vec<u8> = Vec::new();
            i.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"0201fc"});
        }
    }
}
