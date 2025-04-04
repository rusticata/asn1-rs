use crate::*;
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::{fmt, iter::FusedIterator, marker::PhantomData, ops::Shl, str::FromStr};
use displaydoc::Display;
use nom::Input as _;
use num_traits::Num;
use thiserror::Error;

/// An error for OID parsing functions.
#[derive(Clone, Copy, Debug, Display, PartialEq, Eq, Error)]
pub enum OidParseError {
    /// Encoded data length too short
    TooShort,
    /** Signalizes that the first or second component is too large.
     * The first must be within the range 0 to 6 (inclusive).
     * The second component must be less than 40.
     */
    FirstComponentsTooLarge,
    /// a
    ParseIntError,
}

/// Object ID (OID) representation which can be relative or non-relative.
///
/// An example for an OID in string representation is `"1.2.840.113549.1.1.5"`.
///
/// For non-relative OIDs restrictions apply to the first two components.
///
/// This library contains a procedural macro `oid` which can be used to
/// create oids. For example `oid!(1.2.44.233)` or `oid!(rel 44.233)`
/// for relative oids. See the [module documentation](index.html) for more information.
#[derive(Hash, PartialEq, Eq, Clone)]
pub struct Oid<'a> {
    asn1: Cow<'a, [u8]>,
    relative: bool,
}

impl_tryfrom_any!('i @ Oid<'i>);

impl<'i> BerParser<'i> for Oid<'i> {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 8.19.1)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        // Each sub-identifier is represented as a series of (one or more) octets (X.690: 8.19.2)
        if input.is_empty() {
            return Err(BerError::nom_err_input(&input, InnerError::InvalidLength));
        }

        let (rem, data) = input.take_split(input.len());
        Ok((rem, Oid::new(Cow::Borrowed(data.as_bytes2()))))
    }
}

impl<'i> DerParser<'i> for Oid<'i> {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // parsing is similar as for BER
        Self::from_ber_content(header, input)
    }
}

impl CheckDerConstraints for Oid<'_> {
    fn check_constraints(any: &Any) -> Result<()> {
        any.header.assert_primitive()?;
        any.header.length.assert_definite()?;
        Ok(())
    }
}

impl DerAutoDerive for Oid<'_> {}

impl DynTagged for Oid<'_> {
    fn tag(&self) -> Tag {
        if self.relative {
            Tag::RelativeOid
        } else {
            Tag::Oid
        }
    }

    fn accept_tag(tag: Tag) -> bool {
        tag == Tag::Oid || tag == Tag::RelativeOid
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for Oid<'_> {
        type Encoder = BerGenericEncoder;

        fn ber_content_len(&self) -> Length {
            Length::Definite(self.asn1.len())
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write(&self.asn1).map_err(Into::into)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (self.class(), self.constructed(), self.tag())
        }
    }

    impl_toder_from_tober!(LFT 'a, Oid<'a>);
};

fn encode_relative(ids: &'_ [u64]) -> impl Iterator<Item = u8> + '_ {
    ids.iter().flat_map(|id| {
        let bit_count = 64 - id.leading_zeros();
        let octets_needed = ((bit_count + 6) / 7).max(1);
        (0..octets_needed).map(move |i| {
            let flag = if i == octets_needed - 1 { 0 } else { 1 << 7 };
            ((id >> (7 * (octets_needed - 1 - i))) & 0b111_1111) as u8 | flag
        })
    })
}

impl<'a> Oid<'a> {
    /// Create an OID from the ASN.1 DER encoded form. See the [module documentation](index.html)
    /// for other ways to create oids.
    pub const fn new(asn1: Cow<'a, [u8]>) -> Oid<'a> {
        Oid {
            asn1,
            relative: false,
        }
    }

    /// Create a relative OID from the ASN.1 DER encoded form. See the [module documentation](index.html)
    /// for other ways to create relative oids.
    pub const fn new_relative(asn1: Cow<'a, [u8]>) -> Oid<'a> {
        Oid {
            asn1,
            relative: true,
        }
    }

    /// Build an OID from an array of object identifier components.
    /// This method allocates memory on the heap.
    pub fn from(s: &[u64]) -> core::result::Result<Oid<'static>, OidParseError> {
        if s.len() < 2 {
            if s.len() == 1 && s[0] == 0 {
                return Ok(Oid {
                    asn1: Cow::Borrowed(&[0]),
                    relative: false,
                });
            }
            return Err(OidParseError::TooShort);
        }
        if s[0] >= 7 || s[1] >= 40 {
            return Err(OidParseError::FirstComponentsTooLarge);
        }
        let asn1_encoded: Vec<u8> = [(s[0] * 40 + s[1]) as u8]
            .iter()
            .copied()
            .chain(encode_relative(&s[2..]))
            .collect();
        Ok(Oid {
            asn1: Cow::from(asn1_encoded),
            relative: false,
        })
    }

    /// Build a relative OID from an array of object identifier components.
    pub fn from_relative(s: &[u64]) -> core::result::Result<Oid<'static>, OidParseError> {
        if s.is_empty() {
            return Err(OidParseError::TooShort);
        }
        let asn1_encoded: Vec<u8> = encode_relative(s).collect();
        Ok(Oid {
            asn1: Cow::from(asn1_encoded),
            relative: true,
        })
    }

    /// Create a deep copy of the oid.
    ///
    /// This method allocates data on the heap. The returned oid
    /// can be used without keeping the ASN.1 representation around.
    ///
    /// Cloning the returned oid does again allocate data.
    pub fn to_owned(&self) -> Oid<'static> {
        Oid {
            asn1: Cow::from(self.asn1.to_vec()),
            relative: self.relative,
        }
    }

    /// Get the encoded oid without the header.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.asn1.as_ref()
    }

    /// Get the encoded oid without the header.
    #[deprecated(since = "0.2.0", note = "Use `as_bytes` instead")]
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        self.as_bytes()
    }

    /// Get the bytes representation of the encoded oid
    pub fn into_cow(self) -> Cow<'a, [u8]> {
        self.asn1
    }

    /// Convert the OID to a string representation.
    /// The string contains the IDs separated by dots, for ex: "1.2.840.113549.1.1.5"
    #[cfg(feature = "bigint")]
    pub fn to_id_string(&self) -> String {
        let ints: Vec<String> = self.iter_bigint().map(|i| i.to_string()).collect();
        ints.join(".")
    }

    #[cfg(not(feature = "bigint"))]
    /// Convert the OID to a string representation.
    ///
    /// If every arc fits into a u64 a string like "1.2.840.113549.1.1.5"
    /// is returned, otherwise a hex representation.
    ///
    /// See also the "bigint" feature of this crate.
    pub fn to_id_string(&self) -> String {
        if let Some(arcs) = self.iter() {
            let ints: Vec<String> = arcs.map(|i| i.to_string()).collect();
            ints.join(".")
        } else {
            let mut ret = String::with_capacity(self.asn1.len() * 3);
            for (i, o) in self.asn1.iter().enumerate() {
                ret.push_str(&format!("{:02x}", o));
                if i + 1 != self.asn1.len() {
                    ret.push(' ');
                }
            }
            ret
        }
    }

    /// Return an iterator over the sub-identifiers (arcs).
    #[cfg(feature = "bigint")]
    pub fn iter_bigint(&'_ self) -> impl FusedIterator<Item = BigUint> + ExactSizeIterator + '_ {
        SubIdentifierIterator {
            oid: self,
            pos: 0,
            first: false,
            n: PhantomData,
        }
    }

    /// Return an iterator over the sub-identifiers (arcs).
    /// Returns `None` if at least one arc does not fit into `u64`.
    pub fn iter(&'_ self) -> Option<impl FusedIterator<Item = u64> + ExactSizeIterator + '_> {
        // Check that every arc fits into u64
        let bytes = if self.relative {
            &self.asn1
        } else if self.asn1.is_empty() {
            &[]
        } else {
            &self.asn1[1..]
        };
        let max_bits = bytes
            .iter()
            .fold((0usize, 0usize), |(max, cur), c| {
                let is_end = (c >> 7) == 0u8;
                if is_end {
                    (max.max(cur + 7), 0)
                } else {
                    (max, cur + 7)
                }
            })
            .0;
        if max_bits > 64 {
            return None;
        }

        Some(SubIdentifierIterator {
            oid: self,
            pos: 0,
            first: false,
            n: PhantomData,
        })
    }

    pub fn from_ber_relative(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_ber(bytes)?;
        any.header.assert_primitive()?;
        any.header.assert_tag(Tag::RelativeOid)?;
        let asn1 = Cow::Borrowed(any.data.as_bytes2());
        Ok((rem, Oid::new_relative(asn1)))
    }

    pub fn from_der_relative(bytes: &'a [u8]) -> ParseResult<'a, Self> {
        let (rem, any) = Any::from_der(bytes)?;
        any.header.assert_tag(Tag::RelativeOid)?;
        Self::check_constraints(&any)?;
        let asn1 = Cow::Borrowed(any.data.as_bytes2());
        Ok((rem, Oid::new_relative(asn1)))
    }

    /// Returns true if `needle` is a prefix of the OID.
    pub fn starts_with(&self, needle: &Oid) -> bool {
        self.asn1.len() >= needle.asn1.len() && self.asn1.starts_with(needle.as_bytes())
    }
}

trait Repr: Num + Shl<usize, Output = Self> + From<u8> {}
impl<N> Repr for N where N: Num + Shl<usize, Output = N> + From<u8> {}

struct SubIdentifierIterator<'a, N: Repr> {
    oid: &'a Oid<'a>,
    pos: usize,
    first: bool,
    n: PhantomData<&'a N>,
}

impl<N: Repr> Iterator for SubIdentifierIterator<'_, N> {
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        use num_traits::identities::Zero;

        if self.pos == self.oid.asn1.len() {
            return None;
        }
        if !self.oid.relative {
            if !self.first {
                debug_assert!(self.pos == 0);
                self.first = true;
                return Some((self.oid.asn1[0] / 40).into());
            } else if self.pos == 0 {
                self.pos += 1;
                if self.oid.asn1[0] == 0 && self.oid.asn1.len() == 1 {
                    return None;
                }
                return Some((self.oid.asn1[0] % 40).into());
            }
        }
        // decode objet sub-identifier according to the asn.1 standard
        let mut res = <N as Zero>::zero();
        for o in self.oid.asn1[self.pos..].iter() {
            self.pos += 1;
            res = (res << 7) + (o & 0b111_1111).into();
            let flag = o >> 7;
            if flag == 0u8 {
                break;
            }
        }
        Some(res)
    }
}

impl<N: Repr> FusedIterator for SubIdentifierIterator<'_, N> {}

impl<N: Repr> ExactSizeIterator for SubIdentifierIterator<'_, N> {
    fn len(&self) -> usize {
        if self.oid.relative {
            self.oid.asn1.iter().filter(|o| (*o >> 7) == 0u8).count()
        } else if self.oid.asn1.is_empty() {
            0
        } else if self.oid.asn1.len() == 1 {
            if self.oid.asn1[0] == 0 {
                1
            } else {
                2
            }
        } else {
            2 + self.oid.asn1[2..]
                .iter()
                .filter(|o| (*o >> 7) == 0u8)
                .count()
        }
    }
}

impl fmt::Display for Oid<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.relative {
            f.write_str("rel. ")?;
        }
        f.write_str(&self.to_id_string())
    }
}

impl fmt::Debug for Oid<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("OID(")?;
        <Oid as fmt::Display>::fmt(self, f)?;
        f.write_str(")")
    }
}

impl FromStr for Oid<'_> {
    type Err = OidParseError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let v: core::result::Result<Vec<_>, _> = s.split('.').map(|c| c.parse::<u64>()).collect();
        v.map_err(|_| OidParseError::ParseIntError)
            .and_then(|v| Oid::from(&v))
    }
}

/// Helper macro to declare integers at compile-time
///
/// Since the DER encoded oids are not very readable we provide a
/// procedural macro `oid!`. The macro can be used the following ways:
///
/// - `oid!(1.4.42.23)`: Create a const expression for the corresponding `Oid<'static>`
/// - `oid!(rel 42.23)`: Create a const expression for the corresponding relative `Oid<'static>`
/// - `oid!(raw 1.4.42.23)`/`oid!(raw rel 42.23)`: Obtain the DER encoded form as a byte array.
///
/// # Comparing oids
///
/// Comparing a parsed oid to a static oid is probably the most common
/// thing done with oids in your code. The `oid!` macro can be used in expression positions for
/// this purpose. For example
/// ```
/// use asn1_rs::{oid, Oid};
///
/// # let some_oid: Oid<'static> = oid!(1.2.456);
/// const SOME_STATIC_OID: Oid<'static> = oid!(1.2.456);
/// assert_eq!(some_oid, SOME_STATIC_OID)
/// ```
/// To get a relative Oid use `oid!(rel 1.2)`.
///
/// Because of limitations for procedural macros ([rust issue](https://github.com/rust-lang/rust/issues/54727))
/// and constants used in patterns ([rust issue](https://github.com/rust-lang/rust/issues/31434))
/// the `oid` macro can not directly be used in patterns, also not through constants.
/// You can do this, though:
/// ```
/// # use asn1_rs::{oid, Oid};
/// # let some_oid: Oid<'static> = oid!(1.2.456);
/// const SOME_OID: Oid<'static> = oid!(1.2.456);
/// if some_oid == SOME_OID || some_oid == oid!(1.2.456) {
///     println!("match");
/// }
///
/// // Alternatively, compare the DER encoded form directly:
/// const SOME_OID_RAW: &[u8] = &oid!(raw 1.2.456);
/// match some_oid.as_bytes() {
///     SOME_OID_RAW => println!("match"),
///     _ => panic!("no match"),
/// }
/// ```
/// *Attention*, be aware that the latter version might not handle the case of a relative oid correctly. An
/// extra check might be necessary.
#[macro_export]
macro_rules! oid {
    (raw $( $item:literal ).*) => {
        $crate::exports::asn1_rs_impl::encode_oid!( $( $item ).* )
    };
    (raw $items:expr) => {
        $crate::exports::asn1_rs_impl::encode_oid!($items)
    };
    (rel $($item:literal ).*) => {
        $crate::Oid::new_relative($crate::exports::borrow::Cow::Borrowed(
            &$crate::exports::asn1_rs_impl::encode_oid!(rel $( $item ).*),
        ))
    };
    ($($item:literal ).*) => {
        $crate::Oid::new($crate::exports::borrow::Cow::Borrowed(
            &$crate::oid!(raw $( $item ).*),
        ))
    };
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::{BerParser, DerParser, FromDer, Input, Length, Oid, ToDer};
    use hex_literal::hex;

    #[test]
    fn declare_oid() {
        let oid = super::oid! {1.2.840.113549.1};
        assert_eq!(oid.to_string(), "1.2.840.113549.1");
    }

    const OID_RSA_ENCRYPTION: &[u8] = &oid! {raw 1.2.840.113549.1.1.1};
    const OID_EC_PUBLIC_KEY: &[u8] = &oid! {raw 1.2.840.10045.2.1};
    #[allow(clippy::match_like_matches_macro)]
    fn compare_oid(oid: &Oid) -> bool {
        match oid.as_bytes() {
            OID_RSA_ENCRYPTION => true,
            OID_EC_PUBLIC_KEY => true,
            _ => false,
        }
    }

    #[test]
    fn test_compare_oid() {
        let oid = Oid::from(&[1, 2, 840, 113_549, 1, 1, 1]).unwrap();
        assert_eq!(oid, oid! {1.2.840.113549.1.1.1});
        let oid = Oid::from(&[1, 2, 840, 113_549, 1, 1, 1]).unwrap();
        assert!(compare_oid(&oid));
    }

    #[test]
    fn oid_to_der() {
        let oid = super::oid! {1.2.840.113549.1};
        assert_eq!(oid.der_content_len(), Length::Definite(7));
        let v = oid.to_der_vec().expect("could not serialize");
        assert_eq!(&v, &hex! {"06 07 2a 86 48 86 f7 0d 01"});
        let (_, oid2) = Oid::from_der(&v).expect("could not re-parse");
        assert_eq!(&oid, &oid2);
    }

    #[test]
    fn oid_starts_with() {
        const OID_RSA_ENCRYPTION: Oid = oid! {1.2.840.113549.1.1.1};
        const OID_EC_PUBLIC_KEY: Oid = oid! {1.2.840.10045.2.1};
        let oid = super::oid! {1.2.840.113549.1};
        assert!(OID_RSA_ENCRYPTION.starts_with(&oid));
        assert!(!OID_EC_PUBLIC_KEY.starts_with(&oid));
    }

    #[test]
    fn oid_macro_parameters() {
        // Code inspired from https://github.com/rusticata/der-parser/issues/68
        macro_rules! foo {
            ($a:literal $b:literal $c:literal) => {
                super::oid!($a.$b.$c)
            };
        }

        let oid = foo!(1 2 3);
        assert_eq!(oid, oid! {1.2.3});
    }

    #[test]
    fn parse_ber_oid() {
        let input = &hex!("06 09 2a 86 48 86 f7 0d 01 01 05");
        let (rem, result) = Oid::parse_ber(Input::from(input)).expect("parsing failed");
        let expected = Oid::from(&[1, 2, 840, 113_549, 1, 1, 5]).unwrap();
        assert!(rem.is_empty());
        assert_eq!(result, expected);
    }

    #[test]
    fn parse_der_oid() {
        let input = &hex!("06 09 2a 86 48 86 f7 0d 01 01 05");
        let (rem, result) = Oid::parse_der(Input::from(input)).expect("parsing failed");
        let expected = Oid::from(&[1, 2, 840, 113_549, 1, 1, 5]).unwrap();
        assert!(rem.is_empty());
        assert_eq!(result, expected);
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{Oid, ToBer};

        #[test]
        fn tober_oid() {
            let oid = Oid::from(&[1, 2, 840, 113_549, 1, 1, 5]).unwrap();
            let mut v: Vec<u8> = Vec::new();
            oid.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"06 09 2a864886f70d010105"});
        }

        #[test]
        fn tober_rel_oid() {
            let oid = oid!(rel 1.2);
            let mut v: Vec<u8> = Vec::new();
            oid.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"0d 02 0102"});
        }
    }
}
