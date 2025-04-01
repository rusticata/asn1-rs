mod bmpstring;
mod generalstring;
mod graphicstring;
mod ia5string;
mod numericstring;
mod printablestring;
mod str;
mod string;
mod teletexstring;
mod universalstring;
mod utf8string;
mod videotexstring;
mod visiblestring;

pub use bmpstring::*;
pub use generalstring::*;
pub use graphicstring::*;
pub use ia5string::*;
pub use numericstring::*;
pub use printablestring::*;

pub use teletexstring::*;
pub use universalstring::*;
pub use utf8string::*;
pub use videotexstring::*;
pub use visiblestring::*;

/// Base trait for BER string objects and character set validation
///
/// This trait is implemented by several types, and is used to determine if some bytes
/// would be valid for the given type.
///
/// # Example
///
/// ```rust
/// use asn1_rs::{PrintableString, TestValidCharset, VisibleString};
///
/// let bytes: &[u8] = b"abcd*4";
/// let res = PrintableString::test_valid_charset(bytes);
/// assert!(res.is_err());
/// let res = VisibleString::test_valid_charset(bytes);
/// assert!(res.is_ok());
/// ```
pub trait TestValidCharset {
    /// Check character set for this object type.
    fn test_valid_charset(i: &[u8]) -> crate::Result<()>;
}

#[doc(hidden)]
#[macro_export]
macro_rules! asn1_string {
    (IMPL $name:ident, $sname:expr) => {
        #[doc="ASN.1 restricted character string type (`"]
        #[doc = $sname]
        #[doc = "`)"]
        #[derive(Debug, PartialEq, Eq)]
        pub struct $name<'a> {
            pub(crate) data: alloc::borrow::Cow<'a, str>,
        }

        impl<'a> $name<'a> {
            pub const fn new(s: &'a str) -> Self {
                $name {
                    data: alloc::borrow::Cow::Borrowed(s),
                }
            }

            pub fn string(&self) -> String {
                use alloc::string::ToString;
                self.data.to_string()
            }

            /// Return a reference to the raw string, if shared
            ///
            /// Note: unlike `.as_ref()`, this function can return a reference that can
            /// outlive the current object (if the raw data does).
            #[inline]
            pub fn as_raw_str(&self) -> Option<&'a str> {
                match self.data {
                    alloc::borrow::Cow::Borrowed(s) => Some(s),
                    _ => None,
                }
            }

        }

        impl<'a> AsRef<str> for $name<'a> {
            fn as_ref(&self) -> &str {
                &self.data
            }
        }

        impl<'a> From<&'a str> for $name<'a> {
            fn from(s: &'a str) -> Self {
                Self::new(s)
            }
        }

        impl From<String> for $name<'_> {
            fn from(s: String) -> Self {
                Self {
                    data: alloc::borrow::Cow::Owned(s),
                }
            }
        }

        $crate::impl_tryfrom_any!('i @ $name<'i>);

        impl<'i> $crate::BerParser<'i> for $name<'i> {
            type Error = $crate::BerError<$crate::Input<'i>>;

            fn from_ber_content(header: &'_ $crate::Header<'i>, input: $crate::Input<'i>) -> $crate::nom::IResult<$crate::Input<'i>, Self, Self::Error> {
                use alloc::borrow::Cow;
                // Encoding shall either be primitive or constructed (X.690: 8.20)
                let (rem, data) =
                    if !header.constructed() {
                        use $crate::nom::Input as _;

                        let (rem, data) = input.take_split(input.len());
                        (rem, Cow::Borrowed(data.as_bytes2()))
                    } else {
                        let (rem, s) = $crate::parse_ber_segmented::<$crate::OctetString>(header, input, $crate::OCTETSTRING_MAX_RECURSION)?;

                        let s = s.into_cow();
                        (rem, s)
                    };

                let b = data.as_ref();
                <$name>::test_valid_charset(b).map_err(|e|
                    $crate::BerError::nom_err_input(&rem, e.into()))?;

                match data {
                    Cow::Borrowed(b) => {
                        let s = alloc::str::from_utf8(b).map_err(|e|
                            $crate::BerError::nom_err_input(&rem, e.into()))?;
                        let data = Cow::Borrowed(s);
                        Ok((rem, $name { data }))
                    }
                    Cow::Owned(v) => {
                        let s = alloc::string::String::from_utf8(v).map_err(|e|
                            $crate::BerError::nom_err_input(&rem, e.into()))?;

                        let data = Cow::Owned(s);
                        Ok((rem, $name { data }))
                    }
                }
            }
        }

        impl<'i> $crate::DerParser<'i> for $name<'i> {
            type Error = $crate::BerError<$crate::Input<'i>>;

            fn from_der_content(header: &'_ $crate::Header<'i>, input: $crate::Input<'i>) -> $crate::nom::IResult<$crate::Input<'i>, Self, Self::Error> {
                use $crate::BerParser;

                // Encoding shall be primitive (X.690: 10.2)
                header.assert_primitive_input(&input).map_err($crate::nom::Err::Error)?;

                Self::from_ber_content(header, input)
            }
        }

        impl<'a> $crate::CheckDerConstraints for $name<'a> {
            fn check_constraints(any: &$crate::Any) -> $crate::Result<()> {
                any.header.assert_primitive()?;
                Ok(())
            }
        }

        impl $crate::DerAutoDerive for $name<'_> {}

        impl<'a> $crate::Tagged for $name<'a> {
            const TAG: $crate::Tag = $crate::Tag::$name;
        }

        #[cfg(feature = "std")]
        const _: () = {
            use std::io::Write;

            impl $crate::ToBer for $name<'_> {
                type Encoder = $crate::Primitive< { $crate::Tag::$name.0 }>;

                fn ber_content_len(&self) -> $crate::Length {
                    $crate::Length::Definite(self.data.len())
                }

                fn ber_write_content<W: Write>(&self, target: &mut W) -> $crate::SerializeResult<usize> {
                    target.write(self.data.as_bytes()).map_err(Into::into)
                }

                fn ber_tag_info(&self) -> ($crate::Class, bool, $crate::Tag) {
                    use $crate::Tagged;
                    (Self::CLASS, false, Self::TAG)
                }
            }

            impl $crate::ToDer for $name<'_> {
                type Encoder = $crate::Primitive<{ $crate::Tag::$name.0 }>;

                fn der_content_len(&self) -> $crate::Length {
                    $crate::Length::Definite(self.data.len())
                }

                fn der_write_content<W: Write>(&self, target: &mut W) -> $crate::SerializeResult<usize> {
                    target.write(self.data.as_bytes()).map_err(Into::into)
                }

                fn der_tag_info(&self) -> ($crate::Class, bool, $crate::Tag) {
                    use $crate::Tagged;
                    (Self::CLASS, false, Self::TAG)
                }
            }
        };
    };
    ($name:ident) => {
        asn1_string!(IMPL $name, stringify!($name));
    };
}
