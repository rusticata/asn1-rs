/// # BerSequence custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `BerSequence` is a custom derive attribute, to derive a BER `Sequence` parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`TryFrom<Any>`](super::Any), also providing [`FromBer`](super::FromBer)
///   - [`Tagged`](super::Tagged)
///
/// `DerSequence` implies `BerSequence`, and will conflict with this custom derive. Use `BerSequence` when you only want the
/// above traits derived.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromBer`](super::FromBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `BerSequence` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerSequence)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerSequence)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::BerSequence;

/// # BerParserSequence custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `BerParserSequence` is a custom derive attribute, to derive a BER `Sequence` parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`BerParser`](super::BerParser)
///
/// `BerParserSequence` generates only the DER parser, and is compatible with `DerParserSequence`. Use both custom derive
/// attributes if you want both BER and DER parsers.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromBer`](super::FromBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `BerParserSequence` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerParserSequence)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerParserSequence)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::BerParserSequence;

/// # DerSequence custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `DerSequence` is a custom derive attribute, to derive both BER and DER `Sequence` parsers automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`TryFrom<Any>`](super::Any), also providing [`FromBer`](super::FromBer)
///   - [`Tagged`](super::Tagged)
///   - [`CheckDerConstraints`](super::CheckDerConstraints)
///   - [`FromDer`](super::FromDer)
///
/// `DerSequence` implies `BerSequence`, and will conflict with this custom derive.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromDer`](super::FromDer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `DerSequence` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerSequence)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerSequence)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::DerSequence;

/// # DerParserSequence custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `DerParserSequence` is a custom derive attribute, to derive a BER `Sequence` parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`BerParser`](super::BerParser)
///
/// `DerParserSequence` generates only the DER parser, and is compatible with `BerParserSequence`. Use both custom derive
/// attributes if you want both BER and DER parsers.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromBer`](super::FromBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `DerParserSequence` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerParserSequence)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerParserSequence)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::DerParserSequence;

/// # BerSet custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `BerSet` is a custom derive attribute, to derive a BER `Set` parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`TryFrom<Any>`](super::Any), also providing [`FromBer`](super::FromBer)
///   - [`Tagged`](super::Tagged)
///
/// `DerSet` implies `BerSet`, and will conflict with this custom derive. Use `BerSet` when you only want the
/// above traits derived.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromBer`](super::FromBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SET {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `BerSet` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerSet)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerSet)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::BerSet;

/// # BerParserSet custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `BerParserSet` is a custom derive attribute, to derive a BER `Set` parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`BerParser`](super::BerParser)
///
/// `BerParserSet` generates only the DER parser, and is compatible with `DerParserSet`. Use both custom derive
/// attributes if you want both BER and DER parsers.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromBer`](super::FromBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `BerParserSet` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerParserSet)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerParserSet)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::BerParserSet;

/// # DerSet custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `DerSet` is a custom derive attribute, to derive both BER and DER `Set` parsers automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`TryFrom<Any>`](super::Any), also providing [`FromBer`](super::FromBer)
///   - [`Tagged`](super::Tagged)
///   - [`CheckDerConstraints`](super::CheckDerConstraints)
///   - [`FromDer`](super::FromDer)
///
/// `DerSet` implies `BerSet`, and will conflict with this custom derive.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromDer`](super::FromDer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SEt {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `DerSet` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerSet)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerSet)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::DerSet;

/// # DerParserSet custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `DerParserSet` is a custom derive attribute, to derive a BER `Set` parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`BerParser`](super::BerParser)
///
/// `DerParserSet` generates only the DER parser, and is compatible with `BerParserSet`. Use both custom derive
/// attributes if you want both BER and DER parsers.
///
/// Parsers will be automatically derived from struct fields. Every field type must implement the [`FromBer`](super::FromBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `DerParserSet` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerParserSet)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerParserSet)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::DerParserSet;

/// # BerAlias custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `BerAlias` is a custom derive attribute, to derive a BER object parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`TryFrom<Any>`](super::Any), also providing [`FromBer`](super::FromBer)
///   - [`Tagged`](super::Tagged)
///   - [`CheckDerConstraints`](super::CheckDerConstraints)
///   - [`FromDer`](super::FromDer)
///
/// `DerAlias` implies `BerAlias`, and will conflict with this custom derive. Use `BerAlias` when you only want the
/// above traits derived.
///
/// When defining alias, only unnamed (tuple) structs with one field are supported.
///
/// Parser will be automatically derived from the struct field. This field type must implement the `TryFrom<Any>` trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 object:
/// <pre>
/// MyInt ::= INTEGER(0..2^32)
/// </pre>
///
/// Define a structure and add the `BerAlias` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerAlias)]
/// struct S(pub u32);
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerAlias)]
/// #[debug_derive]
/// struct S(pub u32);
/// ```
pub use asn1_rs_derive::BerAlias;

/// # BerParserAlias custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `BerParserAlias` is a custom derive attribute, to derive a DER object parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`DerParser`](super::DerParser)
///
/// When defining alias, only unnamed (tuple) structs with one field are supported.
///
/// Parser will be automatically derived from the struct field. This field type must implement the `BerParser` trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 object:
/// <pre>
/// MyInt ::= INTEGER(0..2^32)
/// </pre>
///
/// Define a structure and add the `BerParserAlias` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerParserAlias)]
/// struct S(pub u32);
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerParserAlias)]
/// #[debug_derive]
/// struct S(pub u32);
/// ```
pub use asn1_rs_derive::BerParserAlias;

/// # DerAlias custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `DerAlias` is a custom derive attribute, to derive a DER object parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`TryFrom<Any>`](super::Any), also providing [`FromBer`](super::FromBer)
///   - [`Tagged`](super::Tagged)
///
/// `DerAlias` implies `BerAlias`, and will conflict with this custom derive.
///
/// When defining alias, only unnamed (tuple) structs with one field are supported.
///
/// Parser will be automatically derived from the struct field. This field type must implement the `TryFrom<Any>` and `FromDer` traits.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 object:
/// <pre>
/// MyInt ::= INTEGER(0..2^32)
/// </pre>
///
/// Define a structure and add the `DerAlias` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerAlias)]
/// struct S(pub u32);
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerAlias)]
/// #[debug_derive]
/// struct S(pub u32);
/// ```
pub use asn1_rs_derive::DerAlias;

/// # DerParserAlias custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `DerParserAlias` is a custom derive attribute, to derive a DER object parser automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`DerParser`](super::DerParser)
///
/// When defining alias, only unnamed (tuple) structs with one field are supported.
///
/// Parser will be automatically derived from the struct field. This field type must implement the `DerParser` trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To parse the following ASN.1 object:
/// <pre>
/// MyInt ::= INTEGER(0..2^32)
/// </pre>
///
/// Define a structure and add the `DerParserAlias` derive:
///
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerParserAlias)]
/// struct S(pub u32);
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerParserAlias)]
/// #[debug_derive]
/// struct S(pub u32);
/// ```
pub use asn1_rs_derive::DerParserAlias;

/// # ToStatic custom derive
///
/// `ToStatic` is a custom derive attribute, to derive the [`ToStatic`] trait automatically from the structure definition.
///
/// ## Example
///
/// ```rust
/// use asn1_rs::ToStatic;
/// use std::borrow::Cow;
///
/// #[derive(ToStatic)]
/// struct S<'a>(pub Cow<'a, str>);
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::ToStatic;
/// use std::borrow::Cow;
///
/// #[derive(ToStatic)]
/// #[debug_derive]
/// struct S<'a>(pub Cow<'a, str>);
/// ```
pub use asn1_rs_derive::ToStatic;

/// # ToBerSequence custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `ToBerSequence` is a custom derive attribute, to derive BER `Sequence` serialization automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`ToBer`](super::ToBer)
///
/// Serialization will be automatically derived from struct fields. Every field type must implement the [`ToBer`](super::ToBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To serialize the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `BerSequence` derive:
///
#[cfg_attr(feature = "std", doc = r#"```rust"#)]
#[cfg_attr(not(feature = "std"), doc = r#"```rust,compile_fail"#)]
/// use asn1_rs::*;
///
/// #[derive(BerSequence, ToBerSequence)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
///
/// let s = S { a: 1, b: 2, c: 3 };
/// let data = s.to_ber_vec().expect("Serialization failed");
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerSequence, ToBerSequence)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::ToBerSequence;

/// # ToDerSequence custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `ToDerSequence` is a custom derive attribute, to derive DER `Sequence` serialization automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`ToDer`](super::ToDer)
///
/// Serialization will be automatically derived from struct fields. Every field type must implement the [`ToDer`](super::ToDer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To serialize the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `DerSequence` derive:
///
#[cfg_attr(feature = "std", doc = r#"```rust"#)]
#[cfg_attr(not(feature = "std"), doc = r#"```rust,compile_fail"#)]
/// use asn1_rs::*;
///
/// #[derive(DerSequence, ToDerSequence)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
///
/// let s = S { a: 1, b: 2, c: 3 };
/// let data = s.to_der_vec().expect("Serialization failed");
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerSequence, ToDerSequence)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::ToDerSequence;

/// # ToBerSet custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `ToBerSet` is a custom derive attribute, to derive BER `Set` serialization automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`ToBer`](super::ToBer)
///
/// Serialization will be automatically derived from struct fields. Every field type must implement the [`ToBer`](super::ToBer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To serialize the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `ToBerSet` derive:
///
#[cfg_attr(feature = "std", doc = r#"```rust"#)]
#[cfg_attr(not(feature = "std"), doc = r#"```rust,compile_fail"#)]
/// use asn1_rs::*;
///
/// #[derive(BerSet, ToBerSet)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
///
/// let s = S { a: 1, b: 2, c: 3 };
/// let data = s.to_ber_vec().expect("Serialization failed");
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(BerSet, ToBerSet)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::ToBerSet;

/// # ToDerSet custom derive
///
/// _This attribute will become deprecated in some time! See [Deprecated attributes](crate::doc::derive#deprecated-attributes)_
///
/// `ToDerSet` is a custom derive attribute, to derive DER `Set` serialization automatically from the structure definition.
/// This attribute will automatically derive implementations for the following traits:
///   - [`ToBer`](super::ToBer)
///
/// Serialization will be automatically derived from struct fields. Every field type must implement the [`ToDer`](super::ToDer) trait.
///
/// See [`derive`](crate::doc::derive) documentation for more examples and documentation.
///
/// ## Examples
///
/// To serialize the following ASN.1 structure:
/// <pre>
/// S ::= SEQUENCE {
///     a INTEGER(0..2^32),
///     b INTEGER(0..2^16),
///     c INTEGER(0..2^16),
/// }
/// </pre>
///
/// Define a structure and add the `ToDerSet` derive:
///
#[cfg_attr(feature = "std", doc = r#"```rust"#)]
#[cfg_attr(not(feature = "std"), doc = r#"```rust,compile_fail"#)]
/// use asn1_rs::*;
///
/// #[derive(DerSet, ToDerSet)]
/// struct S {
///   a: u32,
///   b: u16,
///   c: u16
/// }
///
/// let s = S { a: 1, b: 2, c: 3 };
/// let data = s.to_der_vec().expect("Serialization failed");
/// ```
///
/// ## Debugging
///
/// To help debugging the generated code, the `#[debug_derive]` attribute has been added.
///
/// When this attribute is specified, the generated code will be printed to `stderr` during compilation.
///
/// Example:
/// ```rust
/// use asn1_rs::*;
///
/// #[derive(DerSet, ToDerSet)]
/// #[debug_derive]
/// struct S {
///   a: u32,
/// }
/// ```
pub use asn1_rs_derive::ToDerSet;

#[doc(hidden)]
pub use asn1_rs_derive::BerParserChoice;

#[doc(hidden)]
pub use asn1_rs_derive::DerParserChoice;

/// Derive parsers and encoders for an `enum` representing a `CHOICE`
///
/// See [Choice](crate::doc::derive#choice) section in 'derive' documentation.
pub use asn1_rs_derive::Choice;

/// Derive parsers and encoders for an `enum` representing an `ENUMERATED`
///
/// See [Enumerated](crate::doc::derive#enumerated) section in 'derive' documentation.
pub use asn1_rs_derive::Enumerated;

/// Derive parsers and encoders for a `struct` representing a `SEQUENCE`
///
/// See [BER/DER Sequence](crate::doc::derive#berder-sequence) section in 'derive' documentation.
pub use asn1_rs_derive::Sequence;

/// Derive parsers and encoders for a `struct` representing a `SET`
///
/// See [BER/DER Set](crate::doc::derive#berder-set) section in 'derive' documentation.
pub use asn1_rs_derive::Set;

/// Create an ASN.1 type alias
///
/// See [Alias types](crate::doc::derive#type-alias) section in 'derive' documentation.
pub use asn1_rs_derive::Alias;
