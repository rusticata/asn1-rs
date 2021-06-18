use crate::asn1_string;
use crate::{Error, Result};

asn1_string!(VideotexString);

impl<'a> VideotexString<'a> {
    fn test_string_charset(i: &[u8]) -> Result<()> {
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_visible(b: &u8) -> bool {
            // XXX
            0x20 <= *b && *b <= 0x7f
        }
        if !i.iter().all(is_visible) {
            return Err(Error::StringInvalidCharset);
        }
        Ok(())
    }
}
