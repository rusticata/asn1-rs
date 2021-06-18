use crate::asn1_string;
use crate::{Error, Result};

asn1_string!(NumericString);

impl<'a> NumericString<'a> {
    fn test_string_charset(i: &[u8]) -> Result<()> {
        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_numeric(b: &u8) -> bool {
            matches!(*b, b'0'..=b'9' | b' ')
        }
        if !i.iter().all(is_numeric) {
            return Err(Error::StringInvalidCharset);
        }
        Ok(())
    }
}
