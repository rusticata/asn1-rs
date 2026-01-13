use crate::*;
use alloc::borrow::Cow;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use core::convert::TryFrom;
use nom::{Input as _, Parser};

mod any_set;
mod btreeset;
mod hashset;
mod iterator;
mod set_of;

#[cfg(feature = "std")]
pub use any_set::*;
pub use iterator::*;
pub use set_of::*;

/// The `SET` object is an unordered list of heteregeneous types.
///
/// Sets can usually be of 2 types:
/// - a list of different objects (`SET`, usually parsed as a `struct`)
/// - a list of similar objects (`SET OF`, usually parsed as a `BTreeSet<T>` or `HashSet<T>`)
///
/// The current object covers the former. For the latter, see the [`SetOf`] documentation.
///
/// The `Set` object contains the (*unparsed*) encoded representation of its content. It provides
/// methods to parse and iterate contained objects, or convert the sequence to other types.
///
/// # Building a Set
///
/// To build a DER set:
/// - if the set is composed of objects of the same type, the [`Set::from_iter_to_der`] method can be used
/// - otherwise, the [`ToDer`] trait can be used to create content incrementally
///
#[cfg_attr(feature = "std", doc = r#"```"#)]
#[cfg_attr(not(feature = "std"), doc = r#"```rust,compile_fail"#)]
/// use asn1_rs::{Integer, Set, SerializeResult, ToDer};
///
/// fn build_set<'a>() -> SerializeResult<Set<'a>> {
///     let mut v = Vec::new();
///     // add an Integer object (construct type):
///     let i = Integer::from_u32(4);
///     let _ = i.write_der(&mut v)?;
///     // some primitive objects also implement `ToDer`. A string will be mapped as `Utf8String`:
///     let _ = "abcd".write_der(&mut v)?;
///     // return the set built from the DER content
///     Ok(Set::new(v.into()))
/// }
///
/// let seq = build_set().unwrap();
///
/// ```
///
/// # Examples
///
#[cfg_attr(feature = "std", doc = r#"```"#)]
#[cfg_attr(not(feature = "std"), doc = r#"```rust,compile_fail"#)]
/// use asn1_rs::{Error, Set};
///
/// // build set
/// let it = [2, 3, 4].iter();
/// let set = Set::from_iter_to_der(it).unwrap();
///
/// // `set` now contains the serialized DER representation of the array
///
/// // iterate objects
/// let mut sum = 0;
/// for item in set.der_iter::<u32, Error>() {
///     // item has type `Result<u32>`, since parsing the serialized bytes could fail
///     sum += item.expect("parsing list item failed");
/// }
/// assert_eq!(sum, 9);
///
/// ```
///
/// Note: the above example encodes a `SET OF INTEGER` object, the [`SetOf`] object could
/// be used to provide a simpler API.
///
#[derive(Clone, Debug)]
pub struct Set<'a> {
    /// Serialized DER representation of the set content
    pub content: Cow<'a, [u8]>,
}

impl<'a> Set<'a> {
    /// Build a set, given the provided content
    pub const fn new(content: Cow<'a, [u8]>) -> Self {
        Set { content }
    }

    /// Consume the set and return the content
    #[inline]
    pub fn into_content(self) -> Cow<'a, [u8]> {
        self.content
    }

    /// Return a reference to the raw sequence data, if shared
    ///
    /// Note: unlike `.as_ref()`, this function can return a reference that can
    /// outlive the current object (if the raw data does).
    #[inline]
    pub fn as_raw_slice(&self) -> Option<&'a [u8]> {
        match self.content {
            Cow::Borrowed(s) => Some(s),
            Cow::Owned(_) => None,
        }
    }

    /// Apply the parsing function to the set content, consuming the set
    ///
    /// Note: this function expects the caller to take ownership of content.
    /// In some cases, handling the lifetime of objects is not easy (when keeping only references on
    /// data). Other methods are provided (depending on the use case):
    /// - [`Set::parse`] takes a reference on the set data, but does not consume it,
    /// - [`Set::from_der_and_then`] does the parsing of the set and applying the function
    ///   in one step, ensuring there are only references (and dropping the temporary set).
    pub fn and_then<U, F, E>(self, op: F) -> ParseResult<'a, U, E>
    where
        F: FnOnce(Cow<'a, [u8]>) -> ParseResult<'a, U, E>,
    {
        op(self.content)
    }

    /// Same as [`Set::from_der_and_then`], but using BER encoding (no constraints).
    pub fn from_ber_and_then<U, F, E>(bytes: &'a [u8], op: F) -> ParseResult<'a, U, E>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<'a, U, E>,
        E: From<Error>,
    {
        let (rem, seq) = Set::from_ber(bytes).map_err(Err::convert)?;
        let data = match seq.content {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            Cow::Owned(_) => unreachable!(),
        };
        let (_, res) = op(data)?;
        Ok((rem, res))
    }

    /// Parse a DER set and apply the provided parsing function to content
    ///
    /// After parsing, the set object and header are discarded.
    ///
    /// ```
    /// use asn1_rs::{FromDer, ParseResult, Set};
    ///
    /// // Parse a SET {
    /// //      a INTEGER (0..255),
    /// //      b INTEGER (0..4294967296)
    /// // }
    /// // and return only `(a,b)
    /// fn parser(i: &[u8]) -> ParseResult<'_, (u8, u32)> {
    ///     Set::from_der_and_then(i, |i| {
    ///             let (i, a) = u8::from_der(i)?;
    ///             let (i, b) = u32::from_der(i)?;
    ///             Ok((i, (a, b)))
    ///         }
    ///     )
    /// }
    /// ```
    pub fn from_der_and_then<U, F, E>(bytes: &'a [u8], op: F) -> ParseResult<'a, U, E>
    where
        F: FnOnce(&'a [u8]) -> ParseResult<'a, U, E>,
        E: From<Error>,
    {
        let (rem, seq) = Set::from_der(bytes).map_err(Err::convert)?;
        let data = match seq.content {
            Cow::Borrowed(b) => b,
            // Since 'any' is built from 'bytes', it is borrowed by construction
            Cow::Owned(_) => unreachable!(),
        };
        let (_, res) = op(data)?;
        Ok((rem, res))
    }

    /// Same as [`Set::parse_der_and_then`], but using BER encoding (no constraints).
    pub fn parse_ber_and_then<F, O, E>(input: Input<'a>, op: F) -> IResult<Input<'a>, O, E>
    where
        F: FnOnce(Header<'a>, Input<'a>) -> IResult<Input<'a>, O, E>,
        E: From<BerError<Input<'a>>>,
    {
        let orig_input = input.clone();
        let (rem, any) = Any::parse_ber(input).map_err(Err::convert)?;
        if any.tag() != Tag::Set {
            return Err(Err::Error(
                BerError::unexpected_tag(orig_input, Some(Tag::Set), any.tag()).into(),
            ))?;
        }
        let (_, res) = op(any.header, any.data)?;
        Ok((rem, res))
    }

    /// Parse a DER set and apply the provided parsing function to content
    ///
    /// After parsing, the set object and header are discarded.
    ///
    /// ```
    /// use asn1_rs::{BerError, DerParser, Input, IResult, Set};
    ///
    /// // Parse a SET {
    /// //      a INTEGER (0..255),
    /// //      b INTEGER (0..4294967296)
    /// // }
    /// // and return only `(a,b)
    /// fn parser(i: Input) -> IResult<Input, (u8, u32), BerError<Input>> {
    ///     Set::parse_der_and_then(i, |_, i| {
    ///             let (i, a) = u8::parse_der(i)?;
    ///             let (i, b) = u32::parse_der(i)?;
    ///             Ok((i, (a, b)))
    ///         }
    ///     )
    /// }
    /// ```
    pub fn parse_der_and_then<F, O, E>(input: Input<'a>, op: F) -> IResult<Input<'a>, O, E>
    where
        F: FnOnce(Header<'a>, Input<'a>) -> IResult<Input<'a>, O, E>,
        E: From<BerError<Input<'a>>>,
    {
        let orig_input = input.clone();
        let (rem, any) = Any::parse_der(input).map_err(Err::convert)?;
        if any.tag() != Tag::Set {
            return Err(Err::Error(
                BerError::unexpected_tag(orig_input, Some(Tag::Set), any.tag()).into(),
            ))?;
        }
        let (_, res) = op(any.header, any.data)?;
        Ok((rem, res))
    }

    /// Apply the parsing function to the set content (non-consuming version)
    pub fn parse<F, T, E>(&'a self, mut f: F) -> ParseResult<'a, T, E>
    where
        F: Parser<&'a [u8], Output = T, Error = E>,
    {
        let input: &[u8] = &self.content;
        f.parse(input)
    }

    /// Apply the parsing function to the set content (consuming version)
    ///
    /// Note: to parse and apply a parsing function in one step, use the
    /// [`Set::from_der_and_then`] method.
    ///
    /// # Limitations
    ///
    /// This function fails if the set contains `Owned` data, because the parsing function
    /// takes a reference on data (which is dropped).
    pub fn parse_into<F, T, E>(self, mut f: F) -> ParseResult<'a, T, E>
    where
        F: Parser<&'a [u8], Output = T, Error = E>,
        E: From<Error>,
    {
        match self.content {
            Cow::Borrowed(b) => f.parse(b),
            _ => Err(Err::Error(Error::LifetimeError.into())),
        }
    }

    /// Return an iterator over the set content, attempting to decode objects as BER
    ///
    /// This method can be used when all objects from the set have the same type.
    pub fn ber_iter<T, E>(&'a self) -> SetIterator<'a, T, BerMode, E>
    where
        T: FromBer<'a, E>,
    {
        SetIterator::new(&self.content)
    }

    /// Return an iterator over the set content, attempting to decode objects as DER
    ///
    /// This method can be used when all objects from the set have the same type.
    pub fn der_iter<T, E>(&'a self) -> SetIterator<'a, T, DerMode, E>
    where
        T: FromDer<'a, E>,
    {
        SetIterator::new(&self.content)
    }

    /// Attempt to parse the set as a `SET OF` items (BER), and return the parsed items as a `Vec`.
    pub fn ber_set_of<T, E>(&'a self) -> Result<Vec<T>, E>
    where
        T: FromBer<'a, E>,
        E: From<Error>,
    {
        self.ber_iter().collect()
    }

    /// Attempt to parse the set as a `SET OF` items (DER), and return the parsed items as a `Vec`.
    pub fn der_set_of<T, E>(&'a self) -> Result<Vec<T>, E>
    where
        T: FromDer<'a, E>,
        E: From<Error>,
    {
        self.der_iter().collect()
    }

    /// Attempt to parse the set as a `SET OF` items (BER) (consuming input),
    /// and return the parsed items as a `Vec`.
    ///
    /// Note: if `Self` is an `Owned` object, the data will be duplicated (causing allocations) into separate objects.
    pub fn into_ber_set_of<T, E>(self) -> Result<Vec<T>, E>
    where
        for<'b> T: FromBer<'b, E>,
        E: From<Error>,
        T: ToStatic<Owned = T>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SetIterator::<T, BerMode, E>::new(bytes).collect(),
            Cow::Owned(data) => {
                let v1 = SetIterator::<T, BerMode, E>::new(&data).collect::<Result<Vec<T>, E>>()?;
                let v2 = v1.iter().map(|t| t.to_static()).collect::<Vec<_>>();
                Ok(v2)
            }
        }
    }

    /// Attempt to parse the set as a `SET OF` items (DER) (consuming input),
    /// and return the parsed items as a `Vec`.
    ///
    /// Note: if `Self` is an `Owned` object, the data will be duplicated (causing allocations) into separate objects.
    pub fn into_der_set_of<T, E>(self) -> Result<Vec<T>, E>
    where
        for<'b> T: FromDer<'b, E>,
        E: From<Error>,
        T: ToStatic<Owned = T>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SetIterator::<T, DerMode, E>::new(bytes).collect(),
            Cow::Owned(data) => {
                let v1 = SetIterator::<T, DerMode, E>::new(&data).collect::<Result<Vec<T>, E>>()?;
                let v2 = v1.iter().map(|t| t.to_static()).collect::<Vec<_>>();
                Ok(v2)
            }
        }
    }

    pub fn into_der_set_of_ref<T, E>(self) -> Result<Vec<T>, E>
    where
        T: FromDer<'a, E>,
        E: From<Error>,
    {
        match self.content {
            Cow::Borrowed(bytes) => SetIterator::<T, DerMode, E>::new(bytes).collect(),
            Cow::Owned(_) => Err(Error::LifetimeError.into()),
        }
    }
}

impl ToStatic for Set<'_> {
    type Owned = Set<'static>;

    fn to_static(&self) -> Self::Owned {
        Set {
            content: Cow::Owned(self.content.to_vec()),
        }
    }
}

impl AsRef<[u8]> for Set<'_> {
    fn as_ref(&self) -> &[u8] {
        &self.content
    }
}

impl<'a> TryFrom<Any<'a>> for Set<'a> {
    type Error = Error;

    fn try_from(any: Any<'a>) -> Result<Set<'a>> {
        TryFrom::try_from(&any)
    }
}

impl<'a, 'b> TryFrom<&'b Any<'a>> for Set<'a> {
    type Error = Error;

    fn try_from(any: &'b Any<'a>) -> Result<Set<'a>> {
        any.tag().assert_eq(Self::TAG)?;
        any.header.assert_constructed()?;
        Ok(Set {
            content: Cow::Borrowed(any.data.as_bytes2()),
        })
    }
}

impl<'i> BerParser<'i> for Set<'i> {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.11.1)
        header
            .assert_constructed_input(&input)
            .map_err(Err::Error)?;
        // NOTE: the content is not validated (even the structure)
        let (rem, data) = input.take_split(input.len());
        Ok((
            rem,
            Set {
                content: Cow::Borrowed(data.as_bytes2()),
            },
        ))
    }
}

impl<'i> DerParser<'i> for Set<'i> {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be constructed (X.690: 8.11.1)
        header
            .assert_constructed_input(&input)
            .map_err(Err::Error)?;
        // NOTE: the content is not validated (even the structure)
        let (rem, data) = input.take_split(input.len());
        Ok((
            rem,
            Set {
                content: Cow::Borrowed(data.as_bytes2()),
            },
        ))
    }
}

impl CheckDerConstraints for Set<'_> {
    fn check_constraints(_any: &Any) -> Result<()> {
        Ok(())
    }
}

impl DerAutoDerive for Set<'_> {}

impl Tagged for Set<'_> {
    const CONSTRUCTED: bool = true;
    const TAG: Tag = Tag::Set;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for Set<'_> {
        type Encoder = Constructed;

        fn ber_content_len(&self) -> Length {
            Length::Definite(self.content.len())
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            target.write_all(&self.content)?;
            Ok(self.content.len())
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, true, Self::TAG)
        }
    }

    impl_toder_from_tober!(LFT 'a, Set<'a>);
};

#[cfg(feature = "std")]
impl Set<'_> {
    /// Attempt to create a `Set` from an iterator over serializable objects (to DER)
    ///
    /// # Examples
    ///
    /// ```
    /// use asn1_rs::Set;
    ///
    /// // build set
    /// let it = [2, 3, 4].iter();
    /// let seq = Set::from_iter_to_der(it).unwrap();
    /// ```
    pub fn from_iter_to_der<T, IT>(it: IT) -> SerializeResult<Self>
    where
        IT: Iterator<Item = T>,
        T: ToDer,
        T: Tagged,
    {
        let mut v = Vec::new();
        for item in it {
            let item_v = <T as ToDer>::to_der_vec(&item)?;
            v.extend_from_slice(&item_v);
        }
        Ok(Set {
            content: Cow::Owned(v),
        })
    }
}
