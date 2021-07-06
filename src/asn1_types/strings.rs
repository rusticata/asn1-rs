mod bmpstring;
mod generalstring;
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

pub use self::str::*;
pub use bmpstring::*;
pub use generalstring::*;
pub use ia5string::*;
pub use numericstring::*;
pub use printablestring::*;
pub use string::*;
pub use teletexstring::*;
pub use universalstring::*;
pub use utf8string::*;
pub use videotexstring::*;
pub use visiblestring::*;

#[doc(hidden)]
#[macro_export]
macro_rules! asn1_string {
    (IMPL $name:ident, $sname:expr) => {
        #[doc="ASN.1 restricted character string type (`"]
        #[doc = $sname]
        #[doc = "`)"]
        #[derive(Debug, PartialEq)]
        pub struct $name<'a> {
            pub(crate) data: std::borrow::Cow<'a, str>,
        }

        impl<'a> $name<'a> {
            pub const fn new(s: &'a str) -> Self {
                $name {
                    data: std::borrow::Cow::Borrowed(s),
                }
            }

            pub fn string(&self) -> String {
                self.data.to_string()
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
                    data: std::borrow::Cow::Owned(s),
                }
            }
        }

        impl<'a> std::convert::TryFrom<$crate::Any<'a>> for $name<'a> {
            type Error = $crate::Error;

            fn try_from(any: $crate::Any<'a>) -> $crate::Result<$name<'a>> {
                use crate::traits::Tagged;
                use std::borrow::Cow;
                any.tag().assert_eq(Self::TAG)?;
                Self::test_string_charset(&any.data)?;

                let data = match any.data {
                    Cow::Borrowed(b) => {
                        let s = std::str::from_utf8(b)?;
                        Cow::Borrowed(s)
                    }
                    Cow::Owned(v) => {
                        let s = std::string::String::from_utf8(v)?;
                        Cow::Owned(s)
                    }
                };
                Ok($name { data })
            }
        }

        impl<'a> $crate::CheckDerConstraints for $name<'a> {
            fn check_constraints(any: &$crate::Any) -> $crate::Result<()> {
                any.header.assert_primitive()?;
                Ok(())
            }
        }

        impl<'a> $crate::Tagged for $name<'a> {
            const TAG: $crate::Tag = $crate::Tag::$name;
        }

        impl $crate::ToDer for $name<'_> {
            fn to_der_len(&self) -> Result<usize> {
                let sz = self.data.as_bytes().len();
                if sz < 127 {
                    // 1 (class+tag) + 1 (length) + len
                    Ok(2 + sz)
                } else {
                    // 1 (class+tag) + n (length) + len
                    let n = $crate::Length::Definite(sz).to_der_len()?;
                    Ok(1 + n + sz)
                }
            }

            fn write_der_header(
                &self,
                writer: &mut dyn std::io::Write,
            ) -> $crate::SerializeResult<usize> {
                use $crate::Tagged;
                let header = $crate::Header::new(
                    $crate::Class::Universal,
                    false,
                    Self::TAG,
                    $crate::Length::Definite(self.data.len()),
                );
                header.write_der_header(writer).map_err(Into::into)
            }

            fn write_der_content(
                &self,
                writer: &mut dyn std::io::Write,
            ) -> $crate::SerializeResult<usize> {
                writer.write(self.data.as_bytes()).map_err(Into::into)
            }
        }
    };
    ($name:ident) => {
        asn1_string!(IMPL $name, stringify!($name));
    };
}
