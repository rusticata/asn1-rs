//! Additional documentation: recipes, specific use cases and examples, etc.

#[doc = include_str!("../../doc/RECIPES.md")]
pub mod recipes {}

#[cfg(feature = "std")]
#[doc = include_str!("../../doc/DERIVE.md")]
pub mod derive {}

#[doc = include_str!("../../doc/DEBUG.md")]
pub mod debug {}

#[doc = include_str!("../../doc/ASN1.md")]
pub mod asn1 {}
