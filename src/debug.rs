use crate::ParseResult;

#[macro_export]
macro_rules! debug_eprintln {
    ($msg: expr, $( $args:expr ),* ) => {
        #[cfg(feature = "debug")]
        {
            let s = $msg.to_string().green();
            eprintln!("{} {}", s, format!($($args),*));
        }
    };
}

#[macro_export]
macro_rules! trace_eprintln {
    ($msg: expr, $( $args:expr ),* ) => {
        #[cfg(feature = "trace")]
        {
            let s = $msg.to_string().green();
            eprintln!("{} {}", s, format!($($args),*));
        }
    };
}

#[cfg(feature = "debug")]
fn eprintln_hex_dump(bytes: &[u8], max_len: usize) {
    use core::cmp::min;
    use nom::HexDisplay;

    let m = min(bytes.len(), max_len);
    eprint!("{}", &bytes[..m].to_hex(16));
    if bytes.len() > max_len {
        eprintln!("... <continued>");
    }
}

#[cfg(not(feature = "debug"))]
#[inline]
pub fn trace<'a, T, E, F>(_msg: &str, f: F, input: &'a [u8]) -> ParseResult<'a, T, E>
where
    F: Fn(&'a [u8]) -> ParseResult<'a, T, E>,
{
    f(input)
}

#[cfg(feature = "debug")]
pub fn trace<'a, T, E, F>(msg: &str, f: F, input: &'a [u8]) -> ParseResult<'a, T, E>
where
    F: Fn(&'a [u8]) -> ParseResult<'a, T, E>,
    E: core::fmt::Debug,
{
    use colored::Colorize;

    trace_eprintln!(
        msg,
        "⤷ input (len={}, type={})",
        input.len(),
        core::any::type_name::<T>()
    );
    let res = f(input);
    match &res {
        Ok((_rem, _)) => {
            trace_eprintln!(
                msg,
                "⤶ Parsed {} bytes, {} remaining",
                input.len() - _rem.len(),
                _rem.len()
            );
        }
        Err(e) => {
            debug_eprintln!(msg, "↯ Parsing failed: {}", e.to_string().red());
            eprintln_hex_dump(input, 16);
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::{Any, FromDer};
    use hex_literal::hex;

    #[cfg(feature = "debug")]
    #[test]
    fn debug_from_der_any() {
        assert!(Any::from_der(&hex!("01 01 ff")).is_ok());
    }

    #[cfg(feature = "debug")]
    #[test]
    fn debug_from_der_failures() {
        use crate::Sequence;

        // parsing any failed
        eprintln!("--");
        assert!(u16::from_der(&hex!("ff 00")).is_err());
        // Indefinite length
        eprintln!("--");
        assert!(u16::from_der(&hex!("30 80 00 00")).is_err());
        // DER constraints failed
        eprintln!("--");
        assert!(bool::from_der(&hex!("01 01 7f")).is_err());
        // Incomplete sequence
        eprintln!("--");
        let _ = Sequence::from_der(&hex!("30 81 04 00 00"));
    }

    #[cfg(feature = "debug")]
    #[test]
    fn debug_from_der_sequence() {
        // parsing OK, recursive
        let input = &hex!("30 05 02 03 01 00 01");
        let (rem, result) = <Vec<u32>>::from_der(input).expect("parsing failed");
        assert_eq!(&result, &[65537]);
        assert_eq!(rem, &[]);
    }

    #[cfg(feature = "debug")]
    #[test]
    fn debug_from_der_sequence_of() {
        use crate::SequenceOf;
        // parsing failure (wrong type)
        let input = &hex!("30 03 01 01 00");
        eprintln!("--");
        let _ = <SequenceOf<u32>>::from_der(input).expect_err("parsing should fail");
        eprintln!("--");
        let _ = <Vec<u32>>::from_der(input).expect_err("parsing should fail");
    }

    #[cfg(feature = "debug")]
    #[test]
    fn debug_from_der_set_of() {
        use crate::SetOf;
        use alloc::collections::BTreeSet;

        // parsing failure (wrong type)
        let input = &hex!("31 03 01 01 00");
        eprintln!("--");
        let _ = <SetOf<u32>>::from_der(input).expect_err("parsing should fail");
        eprintln!("--");
        let _ = <BTreeSet<u32>>::from_der(input).expect_err("parsing should fail");
    }

    /// Check that it is possible to implement an error without fmt::Debug
    #[cfg(not(feature = "debug"))]
    #[test]
    fn from_der_error_not_impl_debug() {
        use crate::{CheckDerConstraints, DerAutoDerive};
        use core::convert::TryFrom;

        struct MyError;

        struct A;

        impl CheckDerConstraints for A {
            fn check_constraints(_any: &Any) -> crate::Result<()> {
                Ok(())
            }
        }
        impl<'a> TryFrom<Any<'a>> for A {
            type Error = MyError;

            fn try_from(_value: Any<'a>) -> Result<Self, Self::Error> {
                Ok(A)
            }
        }
        impl DerAutoDerive for A {}

        impl From<crate::Error> for MyError {
            fn from(_value: crate::Error) -> Self {
                Self
            }
        }

        let res = A::from_der(&hex!("02 01 00"));
        assert!(res.is_ok());
    }
}
