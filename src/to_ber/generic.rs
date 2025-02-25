use core::marker::PhantomData;

use super::BerEncoder;

pub trait BerTagEncoder {
    fn write_tag_info<W: std::io::Write>(&self, target: &mut W) -> Result<usize, std::io::Error>;
}

/// Encoder for generic objects
#[allow(missing_debug_implementations)]
pub struct BerGenericEncoder<T: BerTagEncoder> {
    _t: PhantomData<*const T>,
}

impl<T: BerTagEncoder> BerGenericEncoder<T> {
    pub const fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T: BerTagEncoder> Default for BerGenericEncoder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: BerTagEncoder> BerEncoder<T> for BerGenericEncoder<T> {
    fn new() -> Self {
        Self::new()
    }

    fn write_tag_info<W: std::io::Write>(
        &mut self,
        t: &T,
        target: &mut W,
    ) -> Result<usize, std::io::Error> {
        t.write_tag_info(target)
    }
}
