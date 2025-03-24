# BER/DER Custom Derive Attributes

## BER/DER Sequence

### `Sequence`

To derive parsers and encoders for a BER `SEQUENCE` object, add the [`Sequence`] derive attribute to an existing struct. All fields must implement
- [`BerParser`] and [`DerParser`] traits for parsing,
- [`ToBer`] and [`ToDer`] traits for encoding.

By default, all traits are generated (+ [`DynTagged`]).

The `asn1` attribute can be used to control which parsers and encoders are generated (see below).

For ex:

```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, Sequence)]
pub struct S {
    a: u32,
    b: u16,
    c: u16,
}

# let parser = |input| -> IResult<Input, (), BerError<Input>> {
let (rem, result) = S::parse_ber(input)?;
# Ok((rem, ())) };
```

After parsing `input`, any bytes that were leftover and not used to fill val will be returned in `rem`.

When parsing a `SEQUENCE` into a struct, any trailing elements of the `SEQUENCE` that do
not have matching fields in val will not be included in `rem`, as these are considered
valid elements of the `SEQUENCE` and not trailing data.

### Restricting generated parsers and encoders

To control generated code (for ex generate only a `DER` parser), use the `parse` or `encode` items
of the `asn1` attribute.

Each meta item is a string containing a comma-separated list of ASN.1 kinds (`BER` or `DER`)
- if the meta item is absent, it defaults to `"BER,DER"`
- if the meta item is present, code is generated only for the given ASN.1 kinds
- if the meta item is present and empty, no code is generated

| `asn1` meta item | Set of Possible Values | Examples |
| ----- | ----- | ----- |
| `parse` | `""`, `"BER"`, `"DER"` | `#asn1(parse="")`<br />`#asn1(parse="BER")`<br />`#asn1(parse="BER,DER")` |
| `encode` | `""`, `"BER"`, `"DER"` | `#asn1(encode="")`<br />`#asn1(encode="BER")`<br />`#asn1(encode="BER,DER")` |

To generate only the `BER` parser, and no encoder:
```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, Sequence)]
#[asn1(parse="BER", encode="")]
pub struct S {
    a: u32,
    b: u16,
    c: u16,
}
```

## Tagged values

### `EXPLICIT`

There are several ways of parsing tagged values: either using types like [`TaggedExplicit`], or using custom annotations.

Using `TaggedExplicit` works as usual. The only drawback is that the type is visible in the structure, so accessing the value must be done using `.as_ref()` or `.into_inner()`:

```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, DerSequence)]
pub struct S2 {
    a: u16,
}

// test with EXPLICIT Vec
#[derive(Debug, PartialEq, DerSequence)]
pub struct S {
    // a INTEGER
    a: u32,
    // b INTEGER
    b: u16,
    // c [0] EXPLICIT SEQUENCE OF S2
    c: TaggedExplicit<Vec<S2>, Error, 0>,
}

# let parser = |input| -> Result<(), Error> {
let (rem, result) = S::from_ber(input)?;

// Get a reference on c (type is &Vec<S2>)
let ref_c = result.c.as_ref();
# Ok(()) };
```

*Note: tags are context-specific by default. To specify other kind of tags (like `APPLICATION`) use [`TaggedValue`].*

### `tag_explicit`

To "hide" the tag from the parser, the `tag_explicit` attribute is provided. This attribute must specify the tag value (as an integer), and will automatically wrap reading the value with the specified tag.

```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, DerSequence)]
pub struct S {
    // a [0] EXPLICIT INTEGER
    #[tag_explicit(0)]
    a: u16,
}

# let parser = |input| -> Result<(), Error> {
let (rem, result) = S::from_ber(input)?;
# Ok(()) };
```

This method handles transparently the encapsulation and the read of the tagged value.

*Note: tags are context-specific by default. To specify other kind of tags (like `APPLICATION`) add the tag class before the value in the `tag_explicit` attribute.*
For ex: `tag_explicit(APPLICATION 0)` or `tag_explicit(PRIVATE 2)`.

### Tagged optional values

The `optional` custom attribute can be used in addition of `tag_explicit` to specify that the value is `OPTIONAL`.

The type of the annotated field member must be resolvable to `Option`.

```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, DerSequence)]
pub struct S {
    // a [0] EXPLICIT INTEGER OPTIONAL
    #[tag_explicit(0)]
    #[optional]
    a: Option<u16>,
    // b INTEGER
    b: u16,
}

# let parser = |input| -> Result<(), Error> {
let (rem, result) = S::from_ber(input)?;
# Ok(()) };
```

### `IMPLICIT`

Tagged `IMPLICIT` values are handled similarly as for `EXPLICIT`, and can be parsed either using the [`TaggedImplicit`] type, or using the `tag_implicit` custom attribute.

For ex:
```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, DerSequence)]
pub struct S {
    // a [0] IMPLICIT INTEGER OPTIONAL
    #[tag_implicit(0)]
    #[optional]
    a: Option<u16>,
    // b INTEGER
    b: u16,
}

# let parser = |input| -> Result<(), Error> {
let (rem, result) = S::from_ber(input)?;
# Ok(()) };
```

## `OPTIONAL` values (not tagged)

The `optional` custom attribute can be specified to indicate the value is `OPTIONAL`.

```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, DerSequence)]
pub struct S {
    // a INTEGER
    a: u16,
    // b INTEGER OPTIONAL
    #[optional]
    b: Option<u16>,
}

# let parser = |input| -> Result<(), Error> {
let (rem, result) = S::from_ber(input)?;
# Ok(()) };
```

**Important**: there are several limitations to this attribute.

In particular, the parser is eager: when an `OPTIONAL` value of some type is followed by another value (not `OPTIONAL`) of the same type, this can create problem.
If only one value is present, the parser will affect it to the first field, and then raise an error because the second is absent.

Note that this does not concern tagged optional values (unless they have the same tag).

## `DEFAULT`

The `default` custom attribute can be specified to indicate the value has a `DEFAULT` attribute. The value can also be marked as
`OPTIONAL`, but this can be omitted.

Since the value can always be obtained, the type should not be `Option<T>`, but should use `T` directly.

```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, DerSequence)]
#[debug_derive]
pub struct S {
    // a INTEGER
    a: u16,
    // b INTEGER DEFAULT 0
    #[default(0_u16)]
    b: u16,
}

# let parser = |input| -> Result<(), Error> {
let (rem, result) = S::from_ber(input)?;
# Ok(()) };
```

Limitations are the same as for `OPTIONAL` attribute.

## Debugging

To help debugging the generated code, the `#[debug_derive]` attribute has been added.

When this attribute is specified, the generated code will be printed to `stderr` during compilation.

Example:
```rust
use asn1_rs::*;

#[derive(BerSequence)]
#[debug_derive]
struct S {
  a: u32,
}
```

## BER/DER Set

Deriving code for BER/DER `SET` objects is very similar to `SEQUENCE`. Use the [`Set`] custom derive attribute on the structure, and everything else is exactly the same as for sequences (see above for documentation).

Example:
```rust
# use asn1_rs::*;
use std::collections::BTreeSet;

// `Ord` is needed because we will parse as a `BTreeSet` later
#[derive(Debug, Set, PartialEq, Eq, PartialOrd, Ord)]
pub struct S2 {
    a: u16,
}

// test with EXPLICIT Vec
#[derive(Debug, PartialEq, Set)]
pub struct S {
    // a INTEGER
    a: u32,
    // b INTEGER
    b: u16,
    // c [0] EXPLICIT SET OF S2
    c: TaggedExplicit<BTreeSet<S2>, Error, 0>,
}

# let parser = |input| -> IResult<Input, (), BerError<Input>> {
let (rem, result) = S::parse_ber(input)?;

// Get a reference on c (type is &BTreeSet<S2>)
let ref_c = result.c.as_ref();
# Ok((rem, ())) };
```

_Note: The `Sequence` and `Set` attributes cannot be used at the same time on a struct._


# Advanced

## Custom errors (parsers)

Derived parsers can use the `error` attribute to specify the error type of the parser.

The custom error type must implement the following traits, so the derived parsers will transparently convert errors using the [`Into`] trait:
- `From<BerError<Input>>`: convert from errors for primitive/default `asn1_rs` types
- [`nom::error::ParseError`]: common trait for `nom` errors



Example:
```rust
# use asn1_rs::*;
#
#[derive(Debug, PartialEq)]
pub enum MyError {
    NotYetImplemented,
}

impl From<BerError<Input<'_>>> for MyError {
    fn from(_: BerError<Input>) -> Self {
        MyError::NotYetImplemented
    }
}

impl nom::error::ParseError<Input<'_>> for MyError {
    fn from_error_kind(_: Input, _: nom::error::ErrorKind) -> Self {
        MyError::NotYetImplemented
    }
    fn append(_: Input, _: nom::error::ErrorKind, _: Self) -> Self {
        MyError::NotYetImplemented
    }
}

#[derive(Sequence)]
#[error(MyError)]
pub struct T2 {
    pub a: u32,
}
```

## Mapping errors (parsers)

Sometimes, it is necessary to map the returned error to another type, for example when a subparser returns a different error type than the parser's, and the [`Into`] trait cannot be implemented. This is often used in combination with the `error` attribute, but can also be used alone.

The `map_err` attribute can be used to specify a function or closure to map errors. The function signature is `fn (e1: E1) -> E2` with `E1` the parser error type, `E2` the struct parser error type.

Example:
```rust
# use asn1_rs::*;
#
// Here we simply map the error to 'Unsupported'
fn map_to_unsupported(e: BerError<Input>) -> BerError<Input> {
    BerError::new(e.input().clone(), InnerError::Unsupported)
}

#[derive(Sequence)]
pub struct T4 {
    #[map_err(map_to_unsupported)]
    pub a: u32,
}
```

*Note*: when deriving BER and DER parsers, errors paths are different (`TryFrom` returns the error type, while [`FromDer`] returns a [`ParseResult`]). Some code will be inserted by the `map_err` attribute to handle this transparently and keep the same function signature.


# `CHOICE`

The `Choice` derive attribute is used to derive code for an `enum` representing a `CHOICE` object.
Each field represent a possible value.

For convenience, 3 kinds of derive can be generated:
- default ("Untagged"): each variant represent an ASN.1 type
- tagged explicit: each variant represent a type encoded as TAGGED EXPLICIT, with a tag number auto-generated (incremental number of order of appearance of the variant)
- tagged explicit: similar, but with TAGGED IMPLICIT

The `asn1` attribute can be used to control derived code.

### Examples

Default (untagged) `CHOICE`:
```rust
# use asn1_rs::*;
/// MessageType ::= CHOICE
#[derive(Debug, PartialEq, Choice)]
pub enum MessageType<'a> {
    /// text OCTET STRING
    Text(&'a [u8]),
    /// codedNumeric INTEGER
    CodedNumeric(u32),
}

# let parser = |input| -> IResult<Input, (), BerError<Input>> {
let (rem, result) = MessageType::parse_ber(input)?;
# Ok((rem, ())) };
```

`CHOICE` with TAGGED EXPLICIT variants only (using `tagged_explicit` attribute):
```rust
# use asn1_rs::*;
/// Test ::= CHOICE
#[derive(Debug, PartialEq, Choice)]
#[tagged_explicit]
pub enum Test {
    /// age [0] INTEGER
    Age(u32),
    /// index [1] INTEGER
    Index(u32),
}

# let parser = |input| -> IResult<Input, (), BerError<Input>> {
let (rem, result) = Test::parse_ber(input)?;
# Ok((rem, ())) };
```
`CHOICE` with TAGGED IMPLICIT variants only (using `tagged_implicit` attribute):
```rust
# use asn1_rs::*;
/// GeneralName ::= CHOICE
#[derive(Debug, PartialEq, Choice)]
#[tagged_implicit]
pub enum GeneralName<'a> {
    /// otherName [0]  AnotherName
    OtherName(Any<'a>),
    /// rfc822Name [1] IA5String
    Rfc822Name(Ia5String<'a>),
     /// dNSName [2] IA5String
    DNSName(Ia5String<'a>),
    // ...
}

# let parser = |input| -> IResult<Input, (), BerError<Input>> {
let (rem, result) = GeneralName::parse_ber(input)?;
# Ok((rem, ())) };
```


# Type Alias

The [`Alias`] derive attribute is used to derive code for an ASN.1 type alias.
It can only be used on a struct with a single anonymous field.

The `asn1` attribute can be used to control derived code.

### Examples

```rust
# use asn1_rs::*;
/// KeyIdentifier ::= OCTET STRING
#[derive(Debug, PartialEq, Alias)]
pub struct KeyIdentifier<'a>(&'a [u8]);

# let parser = |input| -> IResult<Input, (), BerError<Input>> {
let (rem, result) = KeyIdentifier::parse_ber(input)?;
# Ok((rem, ())) };
```

# Serialization

## BER/DER objects serialization

The [`Sequence`], [`Set`], [`Choice`] and [`Alias`] derive attributes derive parsers and encoders by default, for BER and DER (this can be controlled using the `asn1` attribute).

All struct fields must implement the related traits.

DER serialization example:
```rust
# use asn1_rs::*;
#[derive(Debug, PartialEq, Sequence)]
pub struct S {
    a: u32,
    b: u16,
    c: u16,
}

let s = S { a: 1, b: 2, c: 3 };
let output = s.to_der_vec().expect("serialization failed");
let (_rest, result) = S::parse_ber(Input::from(&output)).expect("parsing failed");
assert_eq!(s, result);
```

# Deprecated attributes

The following attributes are becoming deprecated, and will be marked `deprecated` in the next release.

NOTE: they are not yet marked as such to leave some time for transition, and ensure every feature has been ported to the new attributes.

- [`BerParserSet`](crate::BerParserSet), [`DerParserSet`](crate::DerParserSet), [`ToBerSet`](crate::ToBerSet): replace by `Set`
- [`BerParserSequence`](crate::BerParserSequence), [`DerParserSequence`](crate::DerParserSequence), [`ToBerSequence`](crate::ToBerSequence): replace by `Sequence`

[`DynTagged`]: crate::DynTagged
[`Sequence`]: crate::derive::Sequence
[`Set`]: crate::derive::Set
[`Choice`]: crate::derive::Choice
[`Alias`]: crate::derive::Alias
[`BerParser`]: crate::BerParser
[`DerParser`]: crate::DerParser
[`FromBer`]: crate::FromBer
[`FromDer`]: crate::FromDer
[`ToBer`]: crate::ToBer
[`ToDer`]: crate::ToDer
[`BerSequence`]: crate::BerSequence
[`DerSequence`]: crate::DerSequence
[`BerSet`]: crate::BerSet
[`DerSet`]: crate::DerSet
[`ToDerSequence`]: crate::ToDerSequence
[`ParseResult`]: crate::ParseResult
[`TaggedExplicit`]: crate::TaggedExplicit
[`TaggedImplicit`]: crate::TaggedImplicit
[`TaggedValue`]: crate::TaggedValue
