use super::BerEncoder;

/// Encoder for generic objects
#[allow(missing_debug_implementations)]
pub struct BerGenericEncoder {}

impl BerGenericEncoder {
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for BerGenericEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl BerEncoder for BerGenericEncoder {
    fn new() -> Self {
        Self::new()
    }
}
