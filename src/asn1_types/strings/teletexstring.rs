use crate::{asn1_string, TestValidCharset};
use crate::{Error, Result};
#[cfg(not(feature = "std"))]
use alloc::string::String;

asn1_string!(TeletexString);

impl TestValidCharset for TeletexString<'_> {
    fn test_valid_charset(i: &[u8]) -> Result<()> {
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_visible(b: &u8) -> bool {
            0x20 <= *b && *b <= 0x7f
        }
        if !i.iter().all(is_visible) {
            return Err(Error::StringInvalidCharset);
        }
        Ok(())
    }
}
