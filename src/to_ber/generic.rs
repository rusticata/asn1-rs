use core::marker::PhantomData;

use crate::DynTagged;

use super::BerEncoder;

/// Encoder for generic objects
#[allow(missing_debug_implementations)]
pub struct BerGenericEncoder<T: DynTagged> {
    _t: PhantomData<*const T>,
}

impl<T: DynTagged> BerGenericEncoder<T> {
    pub const fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T: DynTagged> Default for BerGenericEncoder<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: DynTagged> BerEncoder<T> for BerGenericEncoder<T> {
    fn new() -> Self {
        Self::new()
    }

    fn write_tag_info<W: std::io::Write>(
        &mut self,
        t: &T,
        target: &mut W,
    ) -> Result<usize, std::io::Error> {
        self.write_tag_generic(t.class(), t.constructed(), t.tag(), target)
    }
}
