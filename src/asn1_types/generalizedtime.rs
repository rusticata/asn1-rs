use crate::*;
use alloc::format;
#[cfg(not(feature = "std"))]
use alloc::string::String;
use core::fmt;
use nom::{AsBytes, Input as _};
#[cfg(feature = "datetime")]
use time::OffsetDateTime;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GeneralizedTime(pub ASN1DateTime);

impl GeneralizedTime {
    pub const fn new(datetime: ASN1DateTime) -> Self {
        GeneralizedTime(datetime)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        // X.680 section 42 defines a GeneralizedTime as a VisibleString restricted to:
        //
        // a) a string representing the calendar date, as specified in ISO 8601, with a four-digit representation of the
        //    year, a two-digit representation of the month and a two-digit representation of the day, without use of
        //    separators, followed by a string representing the time of day, as specified in ISO 8601, without separators
        //    other than decimal comma or decimal period (as provided for in ISO 8601), and with no terminating Z (as
        //    provided for in ISO 8601); or
        // b) the characters in a) above followed by an upper-case letter Z ; or
        // c) he characters in a) above followed by a string representing a local time differential, as specified in
        //    ISO 8601, without separators.
        let (year, month, day, hour, minute, rem) = match bytes {
            [year1, year2, year3, year4, mon1, mon2, day1, day2, hour1, hour2, min1, min2, rem @ ..] =>
            {
                let year_hi = decode_decimal(Self::TAG, *year1, *year2)?;
                let year_lo = decode_decimal(Self::TAG, *year3, *year4)?;
                let year = (year_hi as u32) * 100 + (year_lo as u32);
                let month = decode_decimal(Self::TAG, *mon1, *mon2)?;
                let day = decode_decimal(Self::TAG, *day1, *day2)?;
                let hour = decode_decimal(Self::TAG, *hour1, *hour2)?;
                let minute = decode_decimal(Self::TAG, *min1, *min2)?;
                (year, month, day, hour, minute, rem)
            }
            _ => return Err(Self::TAG.invalid_value("malformed time string (not yymmddhhmm)")),
        };
        if rem.is_empty() {
            return Err(Self::TAG.invalid_value("malformed time string"));
        }
        // check for seconds
        let (second, rem) = match rem {
            [sec1, sec2, rem @ ..] => {
                let second = decode_decimal(Self::TAG, *sec1, *sec2)?;
                (second, rem)
            }
            _ => (0, rem),
        };
        if month > 12 || day > 31 || hour > 23 || minute > 59 || second > 59 {
            // eprintln!("GeneralizedTime: time checks failed");
            // eprintln!(" month:{}", month);
            // eprintln!(" day:{}", day);
            // eprintln!(" hour:{}", hour);
            // eprintln!(" minute:{}", minute);
            // eprintln!(" second:{}", second);
            return Err(Self::TAG.invalid_value("time components with invalid values"));
        }
        if rem.is_empty() {
            // case a): no fractional seconds part, and no terminating Z
            return Ok(GeneralizedTime(ASN1DateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                None,
                ASN1TimeZone::Undefined,
            )));
        }
        // check for fractional seconds
        let (millisecond, rem) = match rem {
            [b'.' | b',', rem @ ..] => {
                let mut fsecond = 0;
                let mut rem = rem;
                let mut digits = 0;
                for idx in 0..=4 {
                    if rem.is_empty() {
                        if idx == 0 {
                            // dot or comma, but no following digit
                            return Err(Self::TAG.invalid_value(
                                "malformed time string (dot or comma but no digits)",
                            ));
                        }
                        digits = idx;
                        break;
                    }
                    if idx == 4 {
                        return Err(
                            Self::TAG.invalid_value("malformed time string (invalid milliseconds)")
                        );
                    }
                    match rem[0] {
                        b'0'..=b'9' => {
                            // cannot overflow, max 4 digits will be read
                            fsecond = fsecond * 10 + (rem[0] - b'0') as u16;
                        }
                        b'Z' | b'+' | b'-' => {
                            digits = idx;
                            break;
                        }
                        _ => {
                            return Err(Self::TAG.invalid_value(
                                "malformed time string (invalid milliseconds/timezone)",
                            ))
                        }
                    }
                    rem = &rem[1..];
                }
                // fix fractional seconds depending on the number of digits
                // for ex, date "xxxx.3" means 3000 milliseconds, not 3
                let fsecond = match digits {
                    1 => fsecond * 100,
                    2 => fsecond * 10,
                    _ => fsecond,
                };
                (Some(fsecond), rem)
            }
            _ => (None, rem),
        };
        // check timezone
        if rem.is_empty() {
            // case a): fractional seconds part, and no terminating Z
            return Ok(GeneralizedTime(ASN1DateTime::new(
                year,
                month,
                day,
                hour,
                minute,
                second,
                millisecond,
                ASN1TimeZone::Undefined,
            )));
        }
        let tz = match rem {
            [b'Z'] => ASN1TimeZone::Z,
            [b'+', h1, h2, m1, m2] => {
                let hh = decode_decimal(Self::TAG, *h1, *h2)?;
                let mm = decode_decimal(Self::TAG, *m1, *m2)?;
                ASN1TimeZone::Offset(hh as i8, mm as i8)
            }
            [b'-', h1, h2, m1, m2] => {
                let hh = decode_decimal(Self::TAG, *h1, *h2)?;
                let mm = decode_decimal(Self::TAG, *m1, *m2)?;
                ASN1TimeZone::Offset(-(hh as i8), mm as i8)
            }
            _ => return Err(Self::TAG.invalid_value("malformed time string: no time zone")),
        };
        Ok(GeneralizedTime(ASN1DateTime::new(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            tz,
        )))
    }

    /// Return a ISO 8601 combined date and time with time zone.
    #[cfg(feature = "datetime")]
    #[cfg_attr(docsrs, doc(cfg(feature = "datetime")))]
    pub fn utc_datetime(&self) -> Result<OffsetDateTime> {
        self.0.to_datetime()
    }
}

impl_tryfrom_any!(GeneralizedTime);

impl<'i> BerParser<'i> for GeneralizedTime {
    type Error = BerError<Input<'i>>;

    fn from_ber_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // GeneralizedTime is encoded as a VisibleString (X.680: 42.3) and can be constructed
        // TODO: constructed GeneralizedTime not supported
        if header.is_constructed() {
            return Err(BerError::nom_err_input(&input, InnerError::Unsupported));
        }

        fn is_visible(b: u8) -> bool {
            (0x20..=0x7f).contains(&b)
        }
        if !input.iter_elements().all(is_visible) {
            return Err(BerError::nom_err_input(
                &input,
                InnerError::StringInvalidCharset,
            ));
        }

        let (rem, data) = input.take_split(input.len());
        let time = GeneralizedTime::from_bytes(data.as_bytes2())
            .map_err(|e| BerError::nom_err_input(&data, e.into()))?;
        Ok((rem, time))
    }
}

impl<'i> DerParser<'i> for GeneralizedTime {
    type Error = BerError<Input<'i>>;

    fn from_der_content(
        header: &'_ Header<'i>,
        input: Input<'i>,
    ) -> IResult<Input<'i>, Self, Self::Error> {
        // Encoding shall be primitive (X.690: 10.2)
        header.assert_primitive_input(&input).map_err(Err::Error)?;

        fn is_visible(b: u8) -> bool {
            (0x20..=0x7f).contains(&b)
        }
        if !input.iter_elements().all(is_visible) {
            return Err(BerError::nom_err_input(
                &input,
                InnerError::StringInvalidCharset,
            ));
        }

        let (rem, data) = input.take_split(input.len());

        // X.690 section 11.7.1: The encoding shall terminate with a "Z"
        if data.as_bytes2().last() != Some(&b'Z') {
            return Err(BerError::nom_err_input(
                &data,
                InnerError::DerConstraintFailed(DerConstraint::MissingTimeZone),
            ));
        }
        // The seconds element shall always be present (X.690: 11.7.2)
        // XXX
        // The decimal point element, if present, shall be the point option "." (X.690: 11.7.4)
        if data.as_bytes2().contains(&b',') {
            return Err(BerError::nom_err_input(
                &data,
                InnerError::DerConstraintFailed(DerConstraint::MissingSeconds),
            ));
        }

        let time = GeneralizedTime::from_bytes(data.as_bytes2())
            .map_err(|e| BerError::nom_err_input(&data, e.into()))?;
        Ok((rem, time))
    }
}

impl fmt::Display for GeneralizedTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let dt = &self.0;
        let fsec = match self.0.millisecond {
            Some(v) => format!(".{}", v),
            None => String::new(),
        };
        match dt.tz {
            ASN1TimeZone::Undefined => write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second, fsec
            ),
            ASN1TimeZone::Z => write!(
                f,
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}Z",
                dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second, fsec
            ),
            ASN1TimeZone::Offset(hh, mm) => {
                let (s, hh) = if hh > 0 { ('+', hh) } else { ('-', -hh) };
                write!(
                    f,
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}{}{}{:02}{:02}",
                    dt.year, dt.month, dt.day, dt.hour, dt.minute, dt.second, fsec, s, hh, mm
                )
            }
        }
    }
}

impl CheckDerConstraints for GeneralizedTime {
    fn check_constraints(any: &Any) -> Result<()> {
        // X.690 section 11.7.1: The encoding shall terminate with a "Z"
        if any.data.as_bytes().last() != Some(&b'Z') {
            return Err(Error::DerConstraintFailed(DerConstraint::MissingTimeZone));
        }
        // X.690 section 11.7.2: The seconds element shall always be present.
        // XXX
        // X.690 section 11.7.4: The decimal point element, if present, shall be the point option "."
        if any.data.as_bytes2().contains(&b',') {
            return Err(Error::DerConstraintFailed(DerConstraint::MissingSeconds));
        }
        Ok(())
    }
}

impl DerAutoDerive for GeneralizedTime {}

impl Tagged for GeneralizedTime {
    const TAG: Tag = Tag::GeneralizedTime;
}

#[cfg(feature = "std")]
const _: () = {
    use std::io::Write;

    impl ToBer for GeneralizedTime {
        type Encoder = Primitive<{ Tag::GeneralizedTime.0 }>;

        fn ber_content_len(&self) -> Length {
            // data:
            // - 8 bytes for YYYYMMDD
            // - 6 for hhmmss in DER (X.690 section 11.7.2)
            // - (variable) the fractional part, without trailing zeros, with a point "."
            // - 1 for the character Z in DER (X.690 section 11.7.1)
            // data length: 15 + fractional part
            let num_digits = match self.0.millisecond {
                None => 0,
                Some(v) => 1 + v.to_string().len(),
            };

            Length::Definite(15 + num_digits)
        }

        fn ber_write_content<W: Write>(&self, target: &mut W) -> SerializeResult<usize> {
            let fractional = match self.0.millisecond {
                None => "".to_string(),
                Some(v) => format!(".{}", v),
            };
            let num_digits = fractional.len();
            write!(
                target,
                "{:04}{:02}{:02}{:02}{:02}{:02}{}Z",
                self.0.year,
                self.0.month,
                self.0.day,
                self.0.hour,
                self.0.minute,
                self.0.second,
                fractional,
            )?;
            // write_fmt returns (), see above for length value
            Ok(15 + num_digits)
        }

        fn ber_tag_info(&self) -> (Class, bool, Tag) {
            (Self::CLASS, false, Self::TAG)
        }
    }

    impl_toder_from_tober!(TY GeneralizedTime);
};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use crate::{ASN1TimeZone, DerConstraint, DerParser, GeneralizedTime, InnerError};

    #[test]
    fn parse_der_generalizedtime() {
        let input = &hex!("18 0F 32 30 30 32 31 32 31 33 31 34 32 39 32 33 5A FF");
        let (rem, result) = GeneralizedTime::parse_der(input.into()).expect("parsing failed");
        assert_eq!(rem.as_bytes2(), &[0xff]);
        #[cfg(feature = "datetime")]
        {
            use time::macros::datetime;
            let datetime = datetime! {2002-12-13 14:29:23 UTC};
            assert_eq!(result.utc_datetime(), Ok(datetime));
        }
        let _ = result;
        // local time with fractional seconds (should fail: no 'Z' at end)
        let input = b"\x18\x1019851106210627.3";
        let result = GeneralizedTime::parse_der(input.into()).expect_err("should not parse");
        assert!(matches!(
            result,
            nom::Err::Error(e) if *e.inner() == InnerError::DerConstraintFailed(DerConstraint::MissingTimeZone)

        ));
        // coordinated universal time with fractional seconds
        let input = b"\x18\x1119851106210627.3Z";
        let (rem, result) = GeneralizedTime::parse_der(input.into()).expect("parsing failed");
        assert!(rem.is_empty());
        assert_eq!(result.0.millisecond, Some(300));
        assert_eq!(result.0.tz, ASN1TimeZone::Z);
        #[cfg(feature = "datetime")]
        {
            use time::macros::datetime;
            let datetime = datetime! {1985-11-06 21:06:27.3 UTC};
            assert_eq!(result.utc_datetime(), Ok(datetime));
        }
        #[cfg(feature = "std")]
        let _ = result.to_string();
        // local time with fractional seconds, and with local time 5 hours retarded in relation to coordinated universal time.
        // (should fail: no 'Z' at end)
        let input = b"\x18\x1519851106210627.3-0500";
        let result = GeneralizedTime::parse_der(input.into()).expect_err("should not parse");
        assert!(matches!(
            result,
            nom::Err::Error(e) if *e.inner() == InnerError::DerConstraintFailed(DerConstraint::MissingTimeZone)

        ));
    }

    #[cfg(feature = "std")]
    mod tests_std {
        use hex_literal::hex;

        use crate::{ASN1DateTime, ASN1TimeZone, GeneralizedTime, ToBer};

        #[test]
        fn tober_generalizedtime() {
            // universal time, no millisecond
            let datetime = ASN1DateTime::new(2013, 12, 2, 14, 29, 23, None, ASN1TimeZone::Z);
            let time = GeneralizedTime::new(datetime);
            let mut v: Vec<u8> = Vec::new();
            time.ber_encode(&mut v).expect("serialization failed");
            let expected = &[&hex!("18 0f") as &[u8], b"20131202142923Z"].concat();
            assert_eq!(&v, expected);

            // universal time with millisecond
            let datetime = ASN1DateTime::new(1999, 12, 31, 23, 59, 59, Some(123), ASN1TimeZone::Z);
            let time = GeneralizedTime::new(datetime);
            let mut v: Vec<u8> = Vec::new();
            time.ber_encode(&mut v).expect("serialization failed");
            let expected = &[&hex!("18 13") as &[u8], b"19991231235959.123Z"].concat();
            assert_eq!(&v, expected);
        }
    }
}
