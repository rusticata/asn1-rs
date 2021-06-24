use crate::{Result, Tag};

#[derive(Clone, Copy, Debug)]
pub enum ASN1TimeZone {
    /// No timezone provided
    Undefined,
    /// Coordinated universal time
    Z,
    /// Local zone, with offset to coordinated universal time
    Offset(i8, u16, u16),
}

#[derive(Debug)]
pub struct ASN1DateTime {
    pub year: u32,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub millisecond: Option<u32>,
    pub tz: ASN1TimeZone,
}

impl ASN1DateTime {
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        year: u32,
        month: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        millisecond: Option<u32>,
        tz: ASN1TimeZone,
    ) -> Self {
        ASN1DateTime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            tz,
        }
    }
}

/// Decode 2-digit decimal value
pub(crate) fn decode_decimal(tag: Tag, hi: u8, lo: u8) -> Result<u16> {
    if (b'0'..=b'9').contains(&hi) && (b'0'..=b'9').contains(&lo) {
        Ok((hi - b'0') as u16 * 10 + (lo - b'0') as u16)
    } else {
        Err(tag.invalid_value("expected digit"))
    }
}
