use std::io::Write;
use std::{io, marker::PhantomData};

use crate::{Length, Tag};

use super::{BerEncoder, ToBer};

/// Encoder for constructed objects, with *Definite* length
#[allow(missing_debug_implementations)]
pub struct ConstructedIndefinite<T> {
    _t: PhantomData<*const T>,
}

impl<T> ConstructedIndefinite<T> {
    pub const fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T> Default for ConstructedIndefinite<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> BerEncoder<T> for ConstructedIndefinite<T> {
    fn new() -> Self {
        ConstructedIndefinite::new()
    }

    fn write_tag_info<W: Write>(&mut self, _t: &T, target: &mut W) -> Result<usize, io::Error> {
        const CONSTRUCTED_BIT: u8 = 0b0010_0000;
        // write tag
        let tag = Tag::Sequence.0; // FIXME: hardcoded
        if tag < 31 {
            // tag is primitive, and uses one byte
            target.write(&[tag as u8 | CONSTRUCTED_BIT])
        } else {
            todo!();
        }
    }
}

/// Wrapper for sequence, to force using Indefinite length when serializing
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IndefiniteVec<T>(pub Vec<T>);

impl<T> ToBer for IndefiniteVec<T>
where
    T: ToBer,
{
    type Encoder = ConstructedIndefinite<IndefiniteVec<T>>;

    fn content_len(&self) -> Length {
        Length::Indefinite
    }

    fn write_content<W: Write>(&self, target: &mut W) -> Result<usize, io::Error> {
        let sz = self.0.iter().try_fold(0, |acc, t| {
            let sz = t.encode(target)?;
            Ok::<_, io::Error>(acc + sz)
        })?;
        // write EndOfContent
        target.write_all(&[0, 0])?;
        Ok(sz + 2)
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    // use nom::HexDisplay;

    use crate::{IndefiniteVec, ToBer};

    #[test]
    fn tober_indefinite_vec() {
        let mut v: Vec<u8> = Vec::new();

        let value = IndefiniteVec(vec![true, false]);
        value.encode(&mut v).expect("serialization failed");
        // eprintln!("encoding for {:?}:\n{}", &value, v.to_hex(16));
        assert_eq!(&v, &hex!("30 80 0101ff 010100 0000"));

        v.clear();
    }
}
