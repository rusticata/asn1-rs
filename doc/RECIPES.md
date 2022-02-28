# Documentation: BER/DER parsing recipes

## `SEQUENCE`

## `EXPLICIT` tagged values

### Parsing `EXPLICIT`, expecting a known tag

If you expect only a specific tag, use `TaggedExplicit`.

For ex, to parse a `[3] EXPLICIT INTEGER`:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = TaggedExplicit::<u32, Error, 0>::from_der(input)?;
// result has type TaggedValue. Use `.as_ref()` or `.into_inner()` 
// to access content
let tag = result.tag();
let class = result.class();
assert_eq!(result.as_ref(), &0);
# Ok(()) };
```

### Specifying the class

`TaggedExplicit` does not check the class, and accepts any class. It expects you to check the class after reading the value.


To specify the class in the parser, use `TaggedValue`:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
// Note: the strange notation (using braces) is required by the compiler to use
// a constant instead of the numeric value.
let (rem, result) = TaggedValue::<u32, Error, Explicit, {Class::CONTEXT_SPECIFIC}, 0>::from_der(input)?;
# Ok(()) };
```

Note that `TaggedExplicit` is a type alias to `TaggedValue`, so the objects are the same.

### Accepting any `EXPLICIT` tag

To parse a value, accepting any class or tag, use `TaggedParser`.

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = TaggedParser::<Explicit, u32>::from_der(input)?;
// result has type TaggedParser. Use `.as_ref()` or `.into_inner()` 
// to access content
let tag = result.tag();
let class = result.class();
assert_eq!(result.as_ref(), &0);
# Ok(()) };
```

### Optional tagged values

To parse optional tagged values, `Option<TaggedExplicit<...>>` can be used:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = Option::<TaggedExplicit::<u32, Error, 0>>::from_der(input)?;
# Ok(()) };
```

The type `OptTaggedExplicit` is also provided as an alias:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = OptTaggedExplicit::<u32, 0>::from_der(input)?;
# Ok(()) };
```

## `IMPLICIT` tagged values

### Parsing `IMPLICIT`, expecting a known tag

If you expect only a specific tag, use `TaggedImplicit`.

For ex, to parse a `[3] EXPLICIT INTEGER`:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = TaggedExplicit::<u32, Error, 0>::from_der(input)?;
// result has type TaggedValue. Use `.as_ref()` or `.into_inner()` 
// to access content
let tag = result.tag();
let class = result.class();
assert_eq!(result.as_ref(), &0);
# Ok(()) };
```

### Specifying the class

`TaggedImplicit` does not check the class, and accepts any class. It expects you to check the class after reading the value.


To specify the class in the parser, use `TaggedValue`:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
// Note: the strange notation (using braces) is required by the compiler to use
// a constant instead of the numeric value.
let (rem, result) = TaggedValue::<u32, Error, Implicit, { Class::CONTEXT_SPECIFIC }, 1>::from_der(input)?;
# Ok(()) };
```

Note that `TaggedImplicit` is a type alias to `TaggedValue`, so the objects are the same.

### Accepting any `IMPLICIT` tag

To parse a value, accepting any class or tag, use `TaggedParser`.

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = TaggedParser::<Implicit, u32>::from_der(input)?;
// result has type TaggedParser. Use `.as_ref()` or `.into_inner()` 
// to access content
let tag = result.tag();
let class = result.class();
assert_eq!(result.as_ref(), &0);
# Ok(()) };
```

### Optional tagged values

To parse optional tagged values, `Option<TaggedImplicit<...>>` can be used:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = Option::<TaggedImplicit::<u32, Error, 0>>::from_der(input)?;
# Ok(()) };
```

The type `OptTaggedImplicit` is also provided as an alias:

```rust
# use asn1_rs::*;
# let parser = |input| -> Result<(), Error> {
let (rem, result) = OptTaggedImplicit::<u32, 0>::from_der(input)?;
# Ok(()) };
```
