use std::io;
use std::io::Write;

use crate::{BerEncoder, Class, Tag};

/// Encoder for constructed objects, with *Definite* length
#[allow(missing_debug_implementations)]
pub struct Constructed {}

impl Constructed {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for Constructed {
    fn default() -> Self {
        Self::new()
    }
}

impl BerEncoder for Constructed {
    fn new() -> Self {
        Constructed::new()
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    // use nom::HexDisplay;

    use crate::ToBer;

    #[test]
    fn tober_constructed_vec() {
        let mut v: Vec<u8> = Vec::new();

        let value = vec![true, false];
        value.ber_encode(&mut v).expect("serialization failed");
        // eprintln!("encoding for {:?}:\n{}", &value, v.to_hex(16));
        assert_eq!(&v, &hex!("30 06 0101ff 010100"));

        v.clear();
    }
}
