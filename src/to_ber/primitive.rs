use std::io;
use std::io::Write;

use crate::{Class, Tag};

use super::BerEncoder;

#[allow(missing_debug_implementations)]
pub struct Primitive<const TAG: u32> {}

impl<const TAG: u32> Primitive<TAG> {
    pub const fn new() -> Self {
        Self {}
    }
}

impl<const TAG: u32> Default for Primitive<TAG> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const TAG: u32> BerEncoder for Primitive<TAG> {
    fn new() -> Self {
        Primitive::new()
    }

    fn write_tag_info<W: Write>(
        &mut self,
        _class: Class,
        _constructed: bool,
        _tag: Tag,
        target: &mut W,
    ) -> Result<usize, io::Error> {
        self.write_tag_generic(Class::Universal, false, Tag(TAG), target)
    }
}
