#![cfg(feature = "std")]

use std::collections::HashSet;

use crate::Any;

/// The `SET` object is an unordered list of heteregeneous types.
///
/// This objects parses all items as `Any`.
///
/// Items in set must be unique. Any attempt to insert an object twice will overwrite the
/// previous object.
/// This is enforced by using a hash function internally.
#[derive(Debug)]
pub struct AnySet<'a> {
    items: HashSet<Any<'a>>,
}
