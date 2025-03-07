use crate::*;
use bitvec::order::Msb0;
use bitvec::slice::BitSlice;
use bitvec::vec::BitVec;

const BITSTRING_MAX_RECURSION: usize = 5;

/// ASN.1 `BITSTRING` type
///
/// This objects owns data (it makes one copy during parsing). Internally, it relies on [`BitVec`].
///
/// Use [`BitString::as_bitslice`] to access content and [`BitString::as_mut_bitslice`] to modify content.
///
/// This type supports constructed objects, but all data segments are appended during parsing
/// (_i.e_ object structure is not kept after parsing).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BitString {
    bitvec: BitVec<u8, Msb0>,
}

impl BitString {
    /// Build a new `BitString`
    ///
    /// # Safety
    /// panic if `unused_bits` is greater than 7 or greater than input length
    pub fn new(unused_bits: u8, s: &[u8]) -> Self {
        let unused_bits = usize::from(unused_bits);
        let mut bitvec = BitVec::from_slice(s);
        assert!(unused_bits < 8 && unused_bits < bitvec.len());
        bitvec.truncate(bitvec.len() - unused_bits);

        BitString { bitvec }
    }

    /// Gets the length of the `BitString` (number of bits)
    pub fn len(&self) -> usize {
        self.bitvec.len()
    }

    /// Tests if the `BitString` is empty
    pub fn is_empty(&self) -> bool {
        self.bitvec.is_empty()
    }

    /// Test if bit `bitnum` is set
    ///
    /// Return false if bit is not set, or if index is outside range.
    pub fn is_set(&self, bitnum: usize) -> bool {
        self.as_bitslice()
            .get(bitnum)
            .map(|bitref| bitref == true)
            .unwrap_or(false)
    }

    /// Return a shared `BitSlice` over the object data.
    pub fn as_bitslice(&self) -> &BitSlice<u8, Msb0> {
        self.bitvec.as_bitslice()
    }

    /// Return a mutable `BitSlice` over the object data.
    pub fn as_mut_bitslice(&mut self) -> &mut BitSlice<u8, Msb0> {
        self.bitvec.as_mut_bitslice()
    }

    /// Return a view over bit-slice bytes
    pub fn as_raw_slice(&self) -> &[u8] {
        self.bitvec.as_raw_slice()
    }
}

impl AsRef<[u8]> for BitString {
    fn as_ref(&self) -> &[u8] {
        self.as_raw_slice()
    }
}

impl_tryfrom_any!(BitString);

impl From<&BitSlice<u8, Msb0>> for BitString {
    fn from(slice: &BitSlice<u8, Msb0>) -> Self {
        let bitvec = BitVec::from_bitslice(slice);
        Self { bitvec }
    }
}

impl<'i> BerParser<'i> for BitString {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall either be primitive or constructed (X.690: 8.6.1)

        if !header.constructed() {
            let (rem, data) = ber_get_content(header, input)?;

            if data.is_empty() {
                return Err(BerError::nom_err_input(&data, InnerError::InvalidLength));
            }

            // safety: data cannot be empty (tested just above)
            let (ignored, bytes) = data.as_bytes2().split_at(1);

            // handle unused bits
            // safety: we have split at index 1
            match ignored[0] {
                ignored @ 0..=7 => {
                    let mut bitvec = BitVec::from_slice(bytes);
                    let new_len = bitvec
                        .len()
                        .checked_sub(usize::from(ignored))
                        .ok_or(BerError::nom_err_input(&data, InnerError::InvalidLength))?;
                    bitvec.truncate(new_len);
                    Ok((rem, Self { bitvec }))
                }
                _ => Err(BerError::nom_err_input(
                    &data,
                    InnerError::invalid_value(Tag::BitString, "Invalid unused bits"),
                )),
            }
        } else {
            // parse_segmented_bitstring(&header, input, BITSTRING_MAX_RECURSION)?
            parse_ber_segmented(header, input, BITSTRING_MAX_RECURSION)
        }
    }
}

impl<'i> DerParser<'i> for BitString {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        <BitString>::from_ber_content(header, input)
    }
}

impl CheckDerConstraints for BitString {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 10.2
        any.header.assert_primitive()?;
        // Check that padding bits are all 0 (X.690 section 11.2.1)
        let s = any.data.as_bytes2();
        match s.len() {
            0 => Err(Error::InvalidLength),
            1 => {
                // X.690 section 11.2.2 Note 2
                if s[0] == 0 {
                    Ok(())
                } else {
                    Err(Error::InvalidLength)
                }
            }
            len => {
                let unused_bits = s[0];
                let last_byte = s[len - 1];
                if last_byte.trailing_zeros() < unused_bits as u32 {
                    return Err(Error::DerConstraintFailed(DerConstraint::UnusedBitsNotZero));
                }

                Ok(())
            }
        }
    }
}

impl DerAutoDerive for BitString {}

impl Tagged for BitString {
    const TAG: Tag = Tag::BitString;
}

impl Appendable for BitString {
    fn append(&mut self, other: &mut Self) {
        self.bitvec.append(&mut other.bitvec);
    }
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for BitString {
        type Encoder = Primitive<{ Tag::BitString.0 }>;

        fn ber_content_len(&self) -> Length {
            let len = 1 + ((self.len() + 7) / 8);
            Length::Definite(len)
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let data = self.as_raw_slice();
            // ignored bits
            let ignored = (8 * data.len()) - self.len();
            target.write_all(&[ignored as u8])?;
            // content
            target.write_all(data)?;

            Ok(1 + data.len())
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(TY BitString);
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{parse_ber_segmented, BerParser, Header, Input};

    use super::BitString;

    #[test]
    fn test_bitstring_is_set() {
        let obj = BitString::new(0, &[0x0f, 0x00, 0x40]);
        assert!(!obj.is_set(0));
        assert!(obj.is_set(7));
        assert!(!obj.is_set(9));
        assert!(obj.is_set(17));
    }

    #[test]
    fn test_bitstring_to_bitvec() {
        let obj = BitString::new(0, &[0x0f, 0x00, 0x40]);
        let bv = obj.as_bitslice();
        assert_eq!(bv.get(0).as_deref(), Some(&false));
        assert_eq!(bv.get(7).as_deref(), Some(&true));
        assert_eq!(bv.get(9).as_deref(), Some(&false));
        assert_eq!(bv.get(17).as_deref(), Some(&true));
    }

    #[test]
    fn test_bitstring_parse_segmented_primitive() {
        //--- Ok: valid data (primitive)
        // example data from X.690 section 8.6.4.2
        let bytes = &hex!("0307 040A3B5F291CD0");
        let (data, header) = Header::parse_ber(Input::from(bytes)).expect("header");
        let (rem, b) = parse_ber_segmented::<BitString>(&header, data, 5).expect("parsing failed");
        assert!(rem.is_empty());
        // compare bitvector length to bitstring bytes, minus ignored bits
        assert_eq!(b.len(), bytes[3..].len() * 8 - usize::from(bytes[2]));

        //--- Fail: invalid length (only ignored bits)
        let bytes = &hex!("0301 04");
        let (data, header) = Header::parse_ber(Input::from(bytes)).expect("header");
        let _ = parse_ber_segmented::<BitString>(&header, data, 5).expect_err("invalid length");

        //--- Fail: invalid length (invalid ignored bits)
        let bytes = &hex!("0302 0901");
        let (data, header) = Header::parse_ber(Input::from(bytes)).expect("header");
        let _ = parse_ber_segmented::<BitString>(&header, data, 5).expect_err("invalid length");
    }

    #[test]
    fn test_bitstring_parse_segmented_constructed() {
        //--- Ok: valid data (primitive)
        // example data from X.690 section 8.6.4.2
        let bytes = &hex!(
            "23 80\
            0303 000A3B\
            0305 045F291CD0\
            00 00"
        );
        let (data, header) = Header::parse_ber(Input::from(bytes)).expect("header");
        let (rem, b) = parse_ber_segmented::<BitString>(&header, data, 5).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(b.len(), 44);

        // Fail: hit recursion limit
        let (data, header) = Header::parse_ber(Input::from(bytes)).expect("header");
        let _ = parse_ber_segmented::<BitString>(&header, data, 1).expect_err("recursion limit");
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use bitvec::{bits, order::Msb0};
        use hex_literal::hex;

        use crate::{BitString, ToBer};

        #[test]
        fn tober_bitstring() {
            let immut = bits![u8, Msb0; 0, 1, 0, 0, 1, 0, 0, 1];
            let bitstring = BitString::from(immut);
            let mut v: Vec<u8> = Vec::new();
            bitstring.ber_encode(&mut v).expect("serialization failed");
            assert_eq!(&v, &hex! {"03020049"});
        }
    }
}
