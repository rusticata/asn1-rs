#![allow(unused_imports)]

use nom::{Err, IResult};

use crate::{Input, ParseResult};

pub(crate) mod macros {
    /// Format and log message at TRACE level, but only if feature `trace` is enabled
    #[allow(unused_macros)]
    macro_rules! log_trace {
        ($fmt: expr) => {
            #[cfg(feature = "trace")]
            {
                log::trace!($fmt);
            }
        };
        ($fmt: expr, $( $args:expr ),*) => {
            #[cfg(feature = "trace")]
            {
                log::trace!($fmt, $($args),*);
            }
        };
    }

    /// Format and log message at ERROR level, but only if feature `debug` is enabled
    #[allow(unused_macros)]
    macro_rules! log_error {
        ($fmt: expr) => {
            #[cfg(feature = "debug")]
            {
                log::error!($fmt);
            }
        };
        ($fmt: expr, $( $args:expr ),*) => {
            #[cfg(feature = "debug")]
            {
                log::error!($fmt, $($args),*);
            }
        };
    }

    /// Format and log message at the specified level, but only if feature `debug` is enabled
    #[allow(unused_macros)]
    macro_rules! debug_log {
        ($lvl: expr, $fmt: expr) => {
            #[cfg(feature = "debug")]
            {
                log::log!($lvl, $fmt);
            }
        };
        ($lvl: expr, $fmt: expr, $( $args:expr ),*) => {
            #[cfg(feature = "debug")]
            {
                log::log!($lvl, $fmt, $($args),*);
            }
        };
    }

    // re-exports for crate
    pub(crate) use {debug_log, log_error, log_trace};
}

use macros::*;

#[cfg(feature = "debug")]
fn log_error_hex_dump(bytes: &[u8], max_len: usize) {
    use core::cmp::min;
    const CHARS: &[u8] = b"0123456789abcdef";

    let m = min(bytes.len(), max_len);
    let mut s = String::with_capacity(3 * m + 1);
    for b in &bytes[..m] {
        s.push(CHARS[(b >> 4) as usize] as char);
        s.push(CHARS[(b & 0xf) as usize] as char);
        s.push(' ');
    }
    s.pop();
    log::error!("{s}");
    if bytes.len() > max_len {
        log::error!("... <continued>");
    }
}

#[cfg(not(feature = "debug"))]
#[inline]
pub fn trace_generic<F, I, O, E>(_msg: &str, _fname: &str, f: F, input: I) -> Result<O, E>
where
    F: Fn(I) -> Result<O, E>,
{
    f(input)
}

#[cfg(feature = "debug")]
pub fn trace_generic<F, I, O, E>(msg: &str, fname: &str, f: F, input: I) -> Result<O, E>
where
    F: Fn(I) -> Result<O, E>,
    E: core::fmt::Display,
{
    log_trace!("{msg} ⤷ {fname}");
    let output = f(input);
    match &output {
        Err(e) => {
            log::error!("{msg} ↯ {fname} failed: {e}");
        }
        _ => {
            log_trace!("{msg} ⤷ {fname}");
        }
    }
    output
}

#[cfg(not(feature = "debug"))]
#[inline]
pub fn trace<'a, T, E, F>(_msg: &str, mut f: F, input: &'a [u8]) -> ParseResult<'a, T, E>
where
    F: FnMut(&'a [u8]) -> ParseResult<'a, T, E>,
{
    f(input)
}

#[cfg(feature = "debug")]
pub fn trace<'a, T, E, F>(msg: &str, mut f: F, input: &'a [u8]) -> ParseResult<'a, T, E>
where
    F: FnMut(&'a [u8]) -> ParseResult<'a, T, E>,
{
    log_trace!(
        "{msg} ⤷ input (len={}, type={})",
        input.len(),
        core::any::type_name::<T>()
    );
    let res = f(input);
    match &res {
        Ok((_rem, _)) => {
            log_trace!(
                "{msg} ⤶ Parsed {} bytes, {} remaining",
                input.len() - _rem.len(),
                _rem.len()
            );
        }
        Err(_) => {
            // NOTE: we do not need to print error, caller should print it
            log::error!("{msg} ↯ Parsing failed at location:");
            log_error_hex_dump(input, 16);
        }
    }
    res
}

#[cfg(not(feature = "debug"))]
#[inline]
pub fn trace_input<'a, T, E, F>(
    _msg: &'a str,
    f: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, T, E>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, T, E>,
    T: 'a,
{
    f
}

/// Call the wrapped function, logging information about input (before) and result (after)
#[cfg(feature = "debug")]
pub fn trace_input<'a, T, E, F>(
    msg: &'a str,
    mut f: F,
) -> impl FnMut(Input<'a>) -> IResult<Input<'a>, T, E>
where
    F: FnMut(Input<'a>) -> IResult<Input<'a>, T, E>,
    T: 'a,
    E: core::fmt::Display,
{
    use nom::Input as _;

    move |input| {
        let start = input.start();
        let bytes = input.as_bytes2();
        log_trace!(
            "{msg} ⤷ input (start={} len={}, type={})",
            start,
            input.len(),
            core::any::type_name::<T>()
        );
        //
        let res = f(input);
        match &res {
            Ok((rem, _)) => {
                debug_assert!(rem.start() > start);
                log_trace!(
                    "{msg} ⤶ (start={}) Parsed {} bytes, {} remaining",
                    start,
                    rem.start() - start,
                    rem.input_len()
                );
            }
            Err(Err::Error(e) | Err::Failure(e)) => {
                log::error!("{msg} ↯ Parsing failed at location {start} with error '{e}':");
                log_error_hex_dump(bytes, 16);
            }
            Err(Err::Incomplete(needed)) => {
                log::error!(
                    "{msg} ↯ Parsing failed at location {start} (missing {:?} bytes):",
                    needed
                );
                log_error_hex_dump(bytes, 16);
            }
        }
        res
    }
}

#[cfg(feature = "debug")]
#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use crate::*;
    use alloc::collections::BTreeSet;
    use hex_literal::hex;

    extern crate env_logger;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn debug_ber_parser_any() {
        //
        init();

        //--- parse_ber_any

        log::debug!("Unit test: parse_ber_any (OK)");
        let input = &hex!("02 01 02 ff ff");
        let (rem, result) = parse_ber_any(Input::from(input)).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        assert_eq!(result.header.tag(), Tag::Integer);

        log::debug!("Unit test: parse_ber_any (Fail: not enough bytes)");
        let input = &hex!("02 08 02 ff ff");
        let _ = parse_ber_any(Input::from(input)).expect_err("not enough bytes");

        //--- Any::parse_ber

        log::debug!("Unit test: Any::parse_ber (OK)");
        let input = &hex!("02 01 02 ff ff");
        let (rem, result) = Any::parse_ber(Input::from(input)).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        assert_eq!(result.header.tag(), Tag::Integer);
    }

    #[test]
    fn debug_der_parser_any() {
        //
        init();

        //--- parse_der_any

        log::debug!("Unit test: parse_der_any (OK)");
        let input = &hex!("02 01 02 ff ff");
        // let (rem, result) = Any::parse_ber(Input::from(input)).expect("parsing failed");
        let (rem, result) = parse_der_any(Input::from(input)).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        assert_eq!(result.header.tag(), Tag::Integer);

        log::debug!("Unit test: parse_ber_any (Fail: not enough bytes)");
        let input = &hex!("02 08 02 ff ff");
        let _ = parse_der_any(Input::from(input)).expect_err("not enough bytes");

        log::debug!("Unit test: parse_ber_any (Fail: indefinite length)");
        let input = &hex!("02 80 00 00 ff");
        let _ = parse_der_any(Input::from(input)).expect_err("indefinite length");

        //--- Any::parse_der

        log::debug!("Unit test: Any::parse_der (OK)");
        let input = &hex!("02 01 02 ff ff");
        let (rem, result) = Any::parse_der(Input::from(input)).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        assert_eq!(result.header.tag(), Tag::Integer);

        //--- Integer::parse_der

        log::debug!("Unit test: Integer::parse_der (OK)");
        let input = &hex!("02 01 02 ff ff");
        let (rem, result) = Integer::parse_der(Input::from(input)).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        assert_eq!(result.as_i32(), Ok(2));

        //--- <u32>::parse_der

        log::debug!("Unit test: <u32>::parse_der (OK)");
        let input = &hex!("02 01 02 ff ff");
        let (rem, result) = <u32>::parse_der(Input::from(input)).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff, 0xff]);
        assert_eq!(result, 2);
    }

    #[test]
    fn debug_from_ber_any() {
        init();

        assert!(Any::from_ber(&hex!("01 01 ff")).is_ok());
    }

    #[test]
    fn debug_from_ber_failures() {
        init();

        // wrong type
        log::debug!("-- wrong type");
        assert!(<Vec<u16>>::from_ber(&hex!("02 01 00")).is_err());
    }

    #[test]
    fn debug_from_ber_sequence_indefinite() {
        let input = &hex!("30 80 02 03 01 00 01 00 00");
        let (rem, result) = Sequence::from_ber(input).expect("parsing failed");
        assert_eq!(result.as_ref(), &input[2..7]);
        assert_eq!(rem, &[]);
        eprintln!("--");
        let (rem, result) = <Vec<u32>>::from_ber(input).expect("parsing failed");
        assert_eq!(&result, &[65537]);
        assert_eq!(rem, &[]);
    }

    #[test]
    fn debug_from_ber_sequence_of() {
        // parsing failure (wrong type)
        let input = &hex!("30 03 01 01 00");
        eprintln!("--");
        let _ = <SequenceOf<u32>>::from_ber(input).expect_err("parsing should fail");
        eprintln!("--");
        let _ = <Vec<u32>>::from_ber(input).expect_err("parsing should fail");
    }

    #[test]
    fn debug_from_ber_u32() {
        assert!(u32::from_ber(&hex!("02 01 01")).is_ok());
    }

    #[test]
    fn debug_from_der_any() {
        assert!(Any::from_der(&hex!("01 01 ff")).is_ok());
    }

    #[test]
    fn debug_from_der_bool() {
        eprintln!("** first test is ok**");
        assert!(<bool>::from_der(&hex!("01 01 ff")).is_ok());
        eprintln!("** second test fails when parsing ANY (eof)**");
        assert!(<bool>::from_der(&hex!("01 02 ff")).is_err());
        eprintln!("** second test fails when checking DER constraints**");
        assert!(<bool>::from_der(&hex!("01 01 f0")).is_err());
        eprintln!("** second test fails during TryFrom**");
        assert!(<bool>::from_der(&hex!("01 02 ff ff")).is_err());
    }

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

    #[test]
    fn debug_from_der_sequence() {
        // parsing OK, recursive
        let input = &hex!("30 08 02 03 01 00 01 02 01 01");
        let (rem, result) = <Vec<u32>>::from_der(input).expect("parsing failed");
        assert_eq!(&result, &[65537, 1]);
        assert_eq!(rem, &[]);
    }

    #[test]
    fn debug_from_der_sequence_fail() {
        // tag is wrong
        let input = &hex!("31 03 01 01 44");
        let _ = <Vec<bool>>::from_der(input).expect_err("parsing should fail");
        // sequence is ok but contraint fails on element
        let input = &hex!("30 03 01 01 44");
        let _ = <Vec<bool>>::from_der(input).expect_err("parsing should fail");
    }

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

    #[test]
    fn debug_from_der_set_fail() {
        // set is ok but contraint fails on element
        let input = &hex!("31 03 01 01 44");
        let _ = <BTreeSet<bool>>::from_der(input).expect_err("parsing should fail");
    }

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
        eprintln!("--");
        let _ = <HashSet<u32>>::from_der(input).expect_err("parsing should fail");
    }

    #[test]
    fn debug_from_der_string_ok() {
        let input = &hex!("0c 0a 53 6f 6d 65 2d 53 74 61 74 65");
        let (rem, result) = Utf8String::from_der(input).expect("parsing failed");
        assert_eq!(result.as_ref(), "Some-State");
        assert_eq!(rem, &[]);
    }

    #[test]
    fn debug_from_der_string_fail() {
        // wrong charset
        let input = &hex!("12 03 41 42 43");
        let _ = NumericString::from_der(input).expect_err("parsing should fail");
    }
}
