use std::io;
use std::io::Write;
use std::marker::PhantomData;

use crate::{BerEncoder, DynTagged};

/// Encoder for constructed objects, with *Definite* length
#[allow(missing_debug_implementations)]
pub struct Constructed<T> {
    _t: PhantomData<*const T>,
}

impl<T> Constructed<T> {
    pub const fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T> Default for Constructed<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> BerEncoder<T> for Constructed<T>
where
    T: DynTagged,
{
    fn new() -> Self {
        Constructed::new()
    }

    fn write_tag_info<W: Write>(&mut self, t: &T, target: &mut W) -> Result<usize, io::Error> {
        const CONSTRUCTED_BIT: u8 = 0b0010_0000;
        // write tag
        let tag = t.tag().0;
        if tag < 31 {
            // tag is primitive, and uses one byte
            target.write(&[tag as u8 | CONSTRUCTED_BIT])
        } else {
            todo!();
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    // use nom::HexDisplay;

    use crate::ToBer;

    #[test]
    fn tober_constructed_vec() {
        let mut v: Vec<u8> = Vec::new();

        let value = vec![true, false];
        value.encode(&mut v).expect("serialization failed");
        // eprintln!("encoding for {:?}:\n{}", &value, v.to_hex(16));
        assert_eq!(&v, &hex!("30 06 0101ff 010100"));

        v.clear();
    }
}
