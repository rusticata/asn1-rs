use std::io::{self, Write};

use crate::{Class, Length, Tag};

/// Common trait for BER encoders
///
/// A BER encoder is an object used to encode the header (full tag including
/// constructed and class) and length
pub trait BerEncoder<T: ?Sized> {
    /// Build a new encoder
    fn new() -> Self;

    /// Write tag, constructed bit, and class to `target`
    // NOTE: mut is here to allow keeping state
    fn write_tag_info<W: Write>(&mut self, t: &T, target: &mut W) -> Result<usize, io::Error>;

    /// This functions writes a full tag (Class, Constructed and Number) to `target`
    ///
    /// Note: this function should not be reimplemented unless implementer has very good reasons!
    fn write_tag_generic<W: Write>(
        &mut self,
        class: Class,
        constructed: bool,
        tag: Tag,
        target: &mut W,
    ) -> Result<usize, io::Error> {
        let class = class as u8;

        const CONSTRUCTED_BIT: u8 = 0b0010_0000;
        let cs = if constructed { CONSTRUCTED_BIT } else { 0 };

        // write tag
        if tag.0 < 31 {
            // tag is primitive, and uses one byte
            let b0 = (class << 6) | cs | (tag.0 as u8);
            target.write(&[b0])
        } else {
            // tag number must be encoded in long form

            // first byte
            let b0 = (class << 6) | cs | 0b1_1111;
            let mut sz = target.write(&[b0])?;

            // now write bytes from right (last) to left
            let mut val = tag.0;

            const BUF_SZ: usize = 8;
            let mut buffer = [0u8; BUF_SZ];
            let mut current_index = BUF_SZ - 1;

            // last encoded byte
            buffer[current_index] = (val & 0x7f) as u8;
            val >>= 7;

            while val > 0 {
                current_index -= 1;
                if current_index == 0 {
                    // return Err(SerializeError::InvalidLength);
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "tag too long"));
                }
                buffer[current_index] = (val & 0x7f) as u8 | 0x80;
                val >>= 7;
            }

            sz += target.write(&buffer[current_index..])?;
            Ok(sz)
        }
    }

    /// Write the length of the encoded object content (without header) to `target`
    fn write_length<W: Write>(
        &mut self,
        _t: &T,
        length: Length,
        target: &mut W,
    ) -> Result<usize, io::Error> {
        const INDEFINITE: u8 = 0b1000_0000;
        match length {
            Length::Indefinite => target.write(&[INDEFINITE]),
            Length::Definite(n) => {
                if n <= 127 {
                    // short form
                    target.write(&[n as u8])
                } else {
                    // long form
                    let b = n.to_be_bytes();
                    // skip leading zeroes
                    // we do not have to test for length, l cannot be 0
                    let mut idx = 0;
                    while b[idx] == 0 {
                        idx += 1;
                    }
                    let b = &b[idx..];
                    // first byte: 0x80 + length of length
                    let b0 = 0x80 | (b.len() as u8);
                    let sz = target.write(&[b0])?;
                    let sz = sz + target.write(b)?;
                    Ok(sz)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerEncoder, Length, Primitive};

    #[test]
    fn tober_write_length() {
        let mut encoder = Primitive::<(), 0>::new();
        let mut v: Vec<u8> = Vec::new();

        // test: Indefinite length
        v.clear();
        encoder
            .write_length(&(), Length::Indefinite, &mut v)
            .expect("serialization failed");
        assert_eq!(&v, &hex!("80"));

        // test: definite length, short-form
        v.clear();
        encoder
            .write_length(&(), Length::Definite(2), &mut v)
            .expect("serialization failed");
        assert_eq!(&v, &hex!("02"));

        // test: definite length, long-form
        v.clear();
        encoder
            .write_length(&(), Length::Definite(300), &mut v)
            .expect("serialization failed");
        assert_eq!(&v, &hex!("82 01 2c"));
    }
}
