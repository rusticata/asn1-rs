use std::io;
use std::io::Write;

use crate::{Class, DynTagged, Length, SerializeError, SerializeResult, Tag};

use super::{BerEncoder, ToBer};

/// Encoder for constructed objects, with *Indefinite* length
#[allow(missing_debug_implementations)]
pub struct ConstructedIndefinite {}

impl ConstructedIndefinite {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for ConstructedIndefinite {
    fn default() -> Self {
        Self::new()
    }
}

impl BerEncoder for ConstructedIndefinite {
    fn new() -> Self {
        ConstructedIndefinite::new()
    }

    fn write_tag_info<W: Write>(
        &mut self,
        class: Class,
        _constructed: bool,
        tag: Tag,
        target: &mut W,
    ) -> Result<usize, io::Error> {
        self.write_tag_generic(class, true, tag, target)
    }
}

/// Wrapper for sequence, to force using Indefinite length when serializing
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct IndefiniteVec<T>(pub Vec<T>);

impl<T> DynTagged for IndefiniteVec<T> {
    fn constructed(&self) -> bool {
        true
    }

    fn tag(&self) -> Tag {
        Tag::Sequence
    }

    fn accept_tag(tag: Tag) -> bool {
        tag == Tag::Sequence
    }
}

impl<T> ToBer for IndefiniteVec<T>
where
    T: ToBer,
{
    type Encoder = ConstructedIndefinite;

    fn content_len(&self) -> Length {
        Length::Indefinite
    }

    fn write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
        let sz = self.0.iter().try_fold(0, |acc, t| {
            let sz = t.encode(target)?;
            Ok::<_, SerializeError>(acc + sz)
        })?;
        // write EndOfContent
        target.write_all(&[0, 0])?;
        Ok(sz + 2)
    }

    fn tag_info(&self) -> (Class, bool, Tag) {
        (self.class(), true, self.tag())
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
