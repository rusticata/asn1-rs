use crate::asn1_string;
use crate::Result;

asn1_string!(Utf8String);

impl<'a> Utf8String<'a> {
    fn test_string_charset(_i: &[u8]) -> Result<()> {
        // no need to check, str::from_utf8 will check bytes
        Ok(())
    }
}
