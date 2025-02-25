use std::io::{self, Write};

use crate::Length;

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
