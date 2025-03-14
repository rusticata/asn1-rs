# ASN.1 types mapping

When using types-based parsing, the usual method is to map the ASN.1 definition of types into `asn1-rs` (Rust) types, and call the matching trait methods on these types.
For generic (not types-based) parsing, other methods are available (but not shown in the following table).

The following table describes how to declare and use ASN.1 types in `asn1-rs`. Some types will have a lifetime, as they try to borrow the input and avoid useless copies.

<table>
<tr>
<th>ASN.1 type</th>
<th>Example</th>
<th>`asn1-rs` type(s)</th>
</tr>

<tr>
<td>BIT STRING</td>
<td>

```asn
My-Type ::= BIT STRING
```

</td>	
<td>

```rust
use asn1_rs::BitString;
type MyType = BitString;
```

</td>
</tr>

<tr>
<td>BOOLEAN</td>
<td>

```asn
My-Type ::= BOOLEAN
```

</td>	
<td>

```rust
type MyType = bool;
```
or
```rust
use asn1_rs::Boolean;
type MyType = Boolean;
```

</td>
</tr>

<tr>
<td>CHOICE</td>
<td>

```asn
My-Type ::= CHOICE {
    text         OCTET STRING,
    codedNumeric INTEGER}
```

</td>	
<td>

_Not yet supported_

</td>
</tr>

<tr>
<td>ENUMERATED</td>
<td>

```asn
My-Type ::= ENUMERATED { a, b, c }
```

</td>	
<td>

Value is parsed as `Enumerated`, which contains the index of the value in the type definition.
```rust
use asn1_rs::Enumerated;
type MyType = Enumerated;

let val = Enumerated::new(2);
```

</td>
</tr>

<tr>
<td>INTEGER</td>
<td>

```asn
UInt8 ::= INTEGER (0..255)
UInt32 ::= INTEGER (0..4294967295)
Int32 ::= INTEGER (-2147483648..2147483647)
```

</td>	
<td>

If integer has known constraints (sign / max), native types can be used:
```rust
type UInt8 = u8;
type UInt32 = u32;
type Int32 = i32;
```

</td>
</tr>

<tr>
<td>INTEGER</td>
<td>

```asn
My-Type ::= INTEGER
```

</td>	
<td>

Variable-length integer:
```rust
use asn1_rs::Integer;
type MyType<'a> = Integer<'a>;

let value = Integer::from(4);
```

</td>
</tr>

<tr>
<td>NULL</td>
<td>

```asn
My-Type ::= NULL
```

</td>	
<td>

```rust
type MyType = ();
```
or
```rust
use asn1_rs::Null;
type MyType = Null;
```

</td>
</tr>

<tr>
<td>OBJECT IDENTIFIER</td>
<td>

```asn
My-Type ::= OBJECT IDENTIFIER
```

</td>	
<td>

The `Oid` type is Copy-on-Write:
```rust
use asn1_rs::*;
type T1<'a> = Oid<'a>;
let oid = oid!(1.2.44.233);
```

</td>
</tr>

<tr>
<td>OCTET STRING</td>
<td>

```asn
My-Type ::= OCTET STRING
```

</td>	
<td>

To use a shared reference (zero-copy) on data:
```rust
type T1<'a> = &'a [u8];
```

or, to use a Copy-on-Write (possible owned) type:
```rust
use asn1_rs::OctetString;
type MyType<'a> = OctetString<'a>;
```

</td>
</tr>

<tr>
<td>REAL</td>
<td>

```asn
My-Type ::= REAL
```

</td>	
<td>

```rust
type MyType1 = f32;
type MyType2 = f64;
```
or
```rust
use asn1_rs::Real;
type MyType = Real;
```

</td>
</tr>

<tr>
<td>RELATIVE-OID</td>
<td>

```asn
My-Type ::= RELATIVE-OID
```

</td>	
<td>

Relative object identifiers are also implemented usind `Oid` (Copy-on-Write):
```rust
use asn1_rs::*;
type T1<'a> = Oid<'a>;
let oid = oid!(rel 44.233);
```

</td>
</tr>

<tr>
<td>UTF8String</td>
<td>

```asn
My-Type ::= UTF8String
```

</td>	
<td>

```rust
type MyType1<'a> = &'a str; // zero-copy
type MyType2 = String; // owned version
```
or the Copy-on-Write version:
```rust
use asn1_rs::Utf8String;
type MyType<'a> = Utf8String<'a>;
```

</td>
</tr>

<tr>
<td>Restricted Character Strings</td>
<td>

```asn
String-N ::= NumericString
String-V ::= VisibleString
String-P ::= PrintableString
String-I ::= IA5String
String-G ::= GeneralString
String-Gr ::= GraphicString
String-T ::= TeletexString
String-U ::= TeletexString

String-B ::= BMPString
String-Un ::= UTF8String
```

</td>	
<td>

Copy-on-Write versions:
```rust
use asn1_rs::*;
type StringN<'a> = NumericString<'a>;
type StringV<'a> = VisibleString<'a>;
type StringP<'a> = PrintableString<'a>;
type StringI<'a> = Ia5String<'a>;
type StringT<'a> = TeletexString<'a>;
type StringG<'a> = GeneralString<'a>;
type StringGr<'a> = GraphicString<'a>;
type StringU<'a> = Utf8String<'a>;
```

Owned Versions:
```rust
use asn1_rs::*;
type StringB<'a> = BmpString<'a>;
type StringUn<'a> = UniversalString<'a>;
```

</td>
</tr>

<tr>
<td>Unrestricted Character Strings</td>
<td>

```asn
My-Type ::= CHARACTER STRING
```

</td>	
<td>

_Not Supported_

</td>
</tr>

<tr>
<td>Time</td>
<td>

```asn
Time-U ::= UTCTime
Time-G ::= GeneralizedTime
```

</td>	
<td>

Owned versions:
```rust
use asn1_rs::*;
type TimeU = UtcTime;
type TimeG = GeneralizedTime;
```

</td>
</tr>

<tr>
<td>Other Time representations</td>
<td>

```asn
My-Type1 ::= TIME
My-Type2 ::= DATE
My-Type3 ::= TIME-OF-DAY
My-Type4 ::= DATE-TIME
My-Type5 ::= DURATION
```

</td>	
<td>

_Not Yet Supported_

</td>
</tr>

<tr>
<td>Sequence</td>
<td>

```asn
My-Type ::= SEQUENCE {
    a BOOLEAN,
    b INTEGER,
}
```

</td>	
<td>

Use custom derive:
```rust
use asn1_rs::*;

#[derive(Debug, PartialEq, DerSequence, ToDerSequence)]
struct MyType {
    a: bool,
    b: u32,
}
```

Generic Versions:
```rust
use asn1_rs::*;
type MyType1<'a> = Sequence<'a>; // generic object with unparsed content
type MyType2<'a> = AnySequence<'a>; // generic ordered collection of any type
```

Fixed length version:
```rust
type MyType = (bool, u32); // BerParser etc. are implemented for (T1, T2, ...)
```
_Note_: when parsing a tuple, all subtypes parsers must return the same type of error

</td>
</tr>

<tr>
<td>SequenceOf</td>
<td>

```asn
My-Type ::= SEQUENCE OF INTEGER
```

</td>	
<td>

Generic Version:
```rust
use asn1_rs::*;
type MyType = SequenceOf<u32>;
```

Native types version:
```rust
type MyType = Vec<u32>; // BerParser etc. are implemented for Vec<T>
```

Fixed length version:
```rust
type MyType = [u32; 7]; // BerParser etc. are implemented for [T; N]
```

</td>
</tr>

<tr>
<td>Set</td>
<td>

```asn
My-Type ::= SET {
    a BOOLEAN,
    b INTEGER,
}
```

</td>	
<td>

Use custom derive:
```rust
use asn1_rs::*;

#[derive(Debug, PartialEq, DerSet, ToDerSet)]
struct MyType {
    a: bool,
    b: u32,
}
```

Generic Versions:
```rust
use asn1_rs::*;
type MyType<'a> = Set<'a>; // generic object with unparsed content
```

Generic Versions (requires `std`):
```no_run,ignore
use asn1_rs::*;
type MyType<'a> = AnySet<'a>; // generic ordered collection of any type
```

</td>
</tr>

<tr>
<td>SetOf</td>
<td>

```asn
My-Type ::= SET OF INTEGER
```

</td>	
<td>

Generic Version:
```rust
use asn1_rs::*;
type MyType = SetOf<u32>;
```

Native types versions (requires `std`):
```no_run,ignore
use std::collections::{BTreeSet, HashSet};
type MyType1 = BTreeSet<u32>; // T must implement Ord
type MyType2 = HashSet<u32>; // T must implement Hash + Eq
```

</td>
</tr>

<tr>
<td>Optional and default fields</td>
<td>

```asn
My-Type ::= SEQUENCE {
    a BOOLEAN DEFAULT TRUE,
    b INTEGER OPTIONAL,
    c INTEGER DEFAULT 1
}
```

</td>	
<td>

When parsing a single type:
```rust
type TypeB = Option<u32>;
```

Using custom derive attribute for a `struct`:
```rust
use asn1_rs::*;

#[derive(Debug, PartialEq, DerSequence, ToDerSequence)]
pub struct MyType {
    #[default(true)]
    a: bool,
    #[optional]
    b: Option<u16>,
    #[default(1)]
    c: u16,
}

```

</td>
</tr>

<tr>
<td>Tagged Explicit</td>
<td>

```asn
-- Explicit tags
My-Type1 ::= [1] BOOLEAN
My-Type2 ::= [APPLICATION 2] INTEGER
```

</td>	
<td>

Using tagged types:
```rust
use asn1_rs::*;
type MyType1<'a> = TaggedExplicit<bool, BerError<Input<'a>>, 1>;
type MyType2<'a> = ApplicationExplicit<Integer<'a>, BerError<Input<'a>>, 2>;

let x = MyType1::explicit(true);
let y = MyType2::explicit(Integer::from(4));
```
_Note_: the error type has to be specified in the type declaration.

or using the generic `TaggedValue` type:
```rust
use asn1_rs::*;
type T1<'a> = TaggedValue<
	bool,
	BerError<Input<'a>>,
	Explicit,
	{Class::CONTEXT_SPECIFIC},
	1>;
let x = T1::explicit(true);
```
_Note_: `TaggedValue` is more flexible, but requires more type annotations.

</td>
</tr>

<tr>
<td>Tagged Implicit</td>
<td>

```asn
-- Implicit tags
My-Type1 ::= [1] BOOLEAN
My-Type2 ::= [APPLICATION 2] INTEGER
```

</td>	
<td>

Using tagged types:
```rust
use asn1_rs::*;
type MyType1<'a> = TaggedImplicit<bool, BerError<Input<'a>>, 1>;
type MyType2<'a> = ApplicationImplicit<Integer<'a>, BerError<Input<'a>>, 2>;

let x = MyType1::implicit(true);
let y = MyType2::implicit(Integer::from(4));
```
_Note_: the error type has to be specified in the type declaration.

or using the generic `TaggedValue` type:
```rust
use asn1_rs::*;
type T1<'a> = TaggedValue<
	bool,
	BerError<Input<'a>>,
	Implicit,
	{Class::CONTEXT_SPECIFIC},
	1>;
let x = T1::implicit(true);
```
_Note_: `TaggedValue` is more flexible, but requires more type annotations.

</td>
</tr>

<tr>
<td>ANY</td>
<td>

_Not strictly an ASN.1 type_

</td>	
<td>

```rust
use asn1_rs::Any;
type MyType<'a> = Any<'a>;
```

</td>
</tr>

</table>

In all of the above examples, parsing and encoding functions can be called directly on the generated type.
For example:
```rust
use asn1_rs::{BerParser, Boolean};
type MyType = Boolean;

let input = asn1_rs::Input::from(b"\x01\x01\xff");
let (rem, my_object) = MyType::parse_ber(input).unwrap();

// Note: you can also use Rust primitive types directly:
let input = asn1_rs::Input::from(b"\x01\x01\xff");
let (rem, my_object) = <bool>::parse_ber(input).unwrap();

```