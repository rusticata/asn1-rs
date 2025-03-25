use alloc::vec::Vec;
use core::iter::{Cloned, Enumerate};
use core::ops::Range;
use core::slice::Iter;

use nom::{AsBytes, Needed};

/// BER/DER parser input type
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Input<'a> {
    data: &'a [u8],
    span: Range<usize>,
}

impl<'a> Input<'a> {
    /// Build a new `Input``
    #[inline]
    pub const fn new(data: &'a [u8], span: Range<usize>) -> Self {
        Self { data, span }
    }

    #[inline]
    pub const fn from_slice(data: &'a [u8]) -> Self {
        let span = Range {
            start: 0,
            end: data.len(),
        };
        Self { data, span }
    }

    #[inline]
    pub const fn const_clone(&self) -> Self {
        Self {
            data: self.data,
            span: Range {
                start: self.span.start,
                end: self.span.end,
            },
        }
    }

    #[inline]
    pub const fn span(&self) -> &Range<usize> {
        &self.span
    }

    #[inline]
    pub const fn start(&self) -> usize {
        self.span.start
    }

    #[inline]
    pub const fn end(&self) -> usize {
        self.span.end
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    // see https://users.rust-lang.org/t/returning-data-from-ephemeral-object/125787/1
    #[inline]
    pub const fn as_bytes2(&self) -> &'a [u8] {
        self.data
    }

    #[inline]
    pub const fn into_bytes(self) -> &'a [u8] {
        self.data
    }
}

impl<'a> AsRef<[u8]> for Input<'a> {
    fn as_ref(&self) -> &'a [u8] {
        self.data
    }
}

impl<'a> AsBytes for Input<'a> {
    fn as_bytes(&self) -> &'a [u8] {
        self.data
    }
}

impl<'a> From<&'a [u8]> for Input<'a> {
    fn from(data: &'a [u8]) -> Self {
        Input {
            data,
            span: Range {
                start: 0,
                end: data.len(),
            },
        }
    }
}

impl<'a, const N: usize> From<&'a [u8; N]> for Input<'a> {
    fn from(value: &'a [u8; N]) -> Self {
        let s = value.as_slice();
        Self::from(s)
    }
}

impl<'a> From<&'a Vec<u8>> for Input<'a> {
    fn from(data: &'a Vec<u8>) -> Self {
        Input {
            data,
            span: Range {
                start: 0,
                end: data.len(),
            },
        }
    }
}

impl<'a> nom::Input for Input<'a> {
    type Item = u8;

    type Iter = Cloned<Iter<'a, u8>>;

    type IterIndices = Enumerate<Self::Iter>;

    fn input_len(&self) -> usize {
        self.data.len()
    }

    fn take(&self, index: usize) -> Self {
        let fragment = &self.data[..index];
        let span = Range {
            start: self.span.start,
            end: self.span.start + index,
        };
        Self::new(fragment, span)
    }

    fn take_from(&self, index: usize) -> Self {
        let fragment = &self.data[index..];
        let span = Range {
            start: self.span.start + index,
            end: self.span.end,
        };
        Self::new(fragment, span)
    }

    fn take_split(&self, index: usize) -> (Self, Self) {
        (self.take_from(index), self.take(index))
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.data.position(predicate)
    }

    fn iter_elements(&self) -> Self::Iter {
        self.data.iter().cloned()
    }

    fn iter_indices(&self) -> Self::IterIndices {
        self.iter_elements().enumerate()
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.data.len() >= count {
            Ok(count)
        } else {
            Err(Needed::new(count - self.data.len()))
        }
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Range;

    use hex_literal::hex;
    use nom::Input as _;

    use super::Input;

    #[test]
    fn input_take() {
        let data = &hex!("00 01 02 03 04 05");
        let input = Input::from(data);

        let r = input.take(2);
        assert_eq!(r, Input::new(&data[..2], Range { start: 0, end: 2 }));

        let r = input.take_from(2);
        assert_eq!(r, Input::new(&data[2..], Range { start: 2, end: 6 }));

        let r = input.iter_elements();
        assert!(r.eq(0..6));

        let r = input.iter_indices();
        assert!(r.eq((0..6).enumerate()));
    }
}
