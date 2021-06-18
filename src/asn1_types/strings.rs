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
    ($name:ident) => {
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
    };
}
