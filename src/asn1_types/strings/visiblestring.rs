use crate::{asn1_string, TestValidCharset};
use crate::{Error, Result};
#[cfg(not(feature = "std"))]
use alloc::string::String;

asn1_string!(VisibleString);

impl TestValidCharset for VisibleString<'_> {
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

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{BerParser, DerParser, Input, VisibleString};

    // Example data from X.690: 8.20.5.4
    // Text is "Jones"
    const VISIBLE_STRING_CONSTRUCTED1: &[u8] = &hex!("3a 09 04034A6F6E 04026573");

    // Example data from X.690: 8.20.5.4
    // Text is "Jones"
    const VISIBLE_STRING_CONSTRUCTED2: &[u8] = &hex!("3a 80 04034A6F6E 04026573 0000");

    #[test]
    fn parse_ber_visiblestring() {
        let input = &hex!("1a 03 31 32 33");
        let (rem, result) = VisibleString::parse_ber(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "123");
        // wrong charset
        let input = &hex!("1a 03 41 00 D8"); // 0x00d8 is the encoding of 'Ø'
        let _ = VisibleString::parse_ber(Input::from(input)).expect_err("parsing should fail");
    }

    #[test]
    fn parse_ber_visiblestring_constructed() {
        // Example data from X.690: 8.20.5.4
        let input = Input::from(VISIBLE_STRING_CONSTRUCTED1);
        let (rem, result) = VisibleString::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "Jones");

        let input = Input::from(VISIBLE_STRING_CONSTRUCTED2);
        let (rem, result) = VisibleString::parse_ber(input).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "Jones");
    }

    #[test]
    fn parse_der_visiblestring() {
        let input = &hex!("1a 03 31 32 33");
        let (rem, result) = VisibleString::parse_der(Input::from(input)).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.as_ref(), "123");

        // Fail: wrong charset
        let input = &hex!("1a 03 41 00 D8"); // 0x00d8 is the encoding of 'Ø'
        let _ = VisibleString::parse_der(Input::from(input)).expect_err("parsing should fail");

        // Fail: constructed
        let input = Input::from(VISIBLE_STRING_CONSTRUCTED1);
        let _ = VisibleString::parse_der(input).expect_err("constructed");
    }
}
