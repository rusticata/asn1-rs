#![deny(/*missing_docs,*/
    unstable_features,
    unused_import_braces,
    unused_qualifications,
    // unreachable_pub
)]
#![forbid(unsafe_code)]
#![warn(
/* missing_docs,
rust_2018_idioms,*/
missing_debug_implementations,
)]
// pragmas for doc
#![deny(broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(test(
no_crate_inject,
attr(deny(warnings/*, rust_2018_idioms*/), allow(dead_code, unused_variables))
))]

mod asn1_types;
mod ber;
mod const_int;
mod datetime;
mod error;
mod header;
mod traits;

pub use asn1_types::*;
pub use const_int::*;
pub use datetime::*;
pub use error::*;
pub use header::*;
pub use traits::*;

pub use nom;
pub use nom::{Err, IResult, Needed};
