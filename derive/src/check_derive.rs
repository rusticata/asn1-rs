use syn::{punctuated::Punctuated, DeriveInput, Ident, Meta, Token};

/// Check if the another "FromBer/FromDer" custom attribute is visible in struct attributes
///
/// Return true if no other attribute is visible (_i.e_ this is the last derive attribute FromBer/Der)
///
/// This should be safe wrt incremental compilation (custom derive attributes will always be in the same file).
///
/// Note on order:
///
/// This relies on the fact that `ast.attrs` only sees attributes *below* the current one, so if
/// there are no further `FromBer*` attributes, this means the current is the last.
///
/// See <https://users.rust-lang.org/t/are-attribute-macros-visible-to-each-other/34266>
pub fn check_lastderive_fromber(ast: &DeriveInput) -> bool {
    // negate the return from `.any()`, because it tries to find other derive values
    !ast.attrs.iter().any(|attr| {
        if attr.path().is_ident("derive") {
            let nested = attr
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .expect("Invalid format in 'derive' attribute");
            for meta in nested {
                // we only check for _our_ attributes, which can only be `Path`
                match meta {
                    Meta::Path(path) => {
                        // `is_ident` uses PartialEq, so use a wrapper to check all patterns at once
                        if path.is_ident(&IdentStartsWithOtherDerive) {
                            // another pattern found - NOT last derive
                            return true;
                        }
                    }
                    Meta::List(_) | Meta::NameValue(_) => (),
                }
            }
        }
        // could not find any searched pattern
        false
    })
}

/// Helper struct to match if an ident starts with a pattern (to be used in `Path::is_ident` only!)
struct IdentStartsWithOtherDerive;

const PATTERNS: &[&str] = &[
    "BerSequence",
    "DerSequence",
    "BerParserSequence",
    "DerParserSequence",
];

impl PartialEq<IdentStartsWithOtherDerive> for Ident {
    fn eq(&self, _other: &IdentStartsWithOtherDerive) -> bool {
        let s = self.to_string();
        PATTERNS.iter().any(|p| s.starts_with(p))
    }
}
