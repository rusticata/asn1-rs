# Change Log

## [Unreleased][unreleased]

### Changed/Fixed

### Added

### Thanks

## 0.4.1

Minor fix:
- add missing file in distribution (fix docs.rs build)

## 0.4.0

asn1-rs:

- Add generic error parameter in traits and in types
  - This was added for all types except a few (like `Vec<T>` or `BTreeSet<T>`) due to
    Rust compiler limitations
- Add `DerAutoDerive` trait to control manual/automatic implementation of `FromDer`
  - This allow controlling automatic trait implementation, and providing manual
    implementations of both `FromDer` and `CheckDerConstraints`
- UtcTime: Introduce utc_adjusted_date() to map 2 chars years date to 20/21 centuries date (#9)

derive:

- Add attributes to simplify deriving EXPLICIT, IMPLICIT and OPTIONAL
- Add support for different tag classes (like APPLICATION or PRIVATE)
- Add support for custom errors and mapping errors
- Add support for deriving BER/DER SET
- DerDerive: derive both CheckDerConstraints and FromDer

documentation:

- Add doc modules for recipes and for custom derive attributes
- Add note on trailing bytes being ignored in sequence
- Improve documentation for notation with braces in TaggedValue
- Improve documentation
