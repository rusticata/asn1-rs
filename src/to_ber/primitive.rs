use std::io;
use std::io::Write;
use std::marker::PhantomData;

use super::BerEncoder;

#[allow(missing_debug_implementations)]
pub struct Primitive<T, const TAG: u32> {
    _t: PhantomData<*const T>,
}

impl<T, const TAG: u32> Primitive<T, TAG> {
    pub const fn new() -> Self {
        Self { _t: PhantomData }
    }
}

impl<T, const TAG: u32> Default for Primitive<T, TAG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const TAG: u32> BerEncoder<T> for Primitive<T, TAG> {
    fn new() -> Self {
        Primitive::new()
    }

    fn write_tag_info<W: Write>(&mut self, _t: &T, target: &mut W) -> Result<usize, io::Error> {
        // write tag
        let tag = TAG;
        if tag < 31 {
            // tag is primitive, and uses one byte
            target.write(&[tag as u8])
        } else {
            todo!();
        }
    }
}
