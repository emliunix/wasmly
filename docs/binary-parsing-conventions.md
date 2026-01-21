# General Binary Parsing Conventions

This document outlines general binary parsing conventions and patterns applicable to any binary format, used as foundation for WASM binary parser.

## Core Concepts

### 1. Input Representation

**Convention**: Use byte slices (`&[u8]`) as input for parser functions

```rust
type Input<'a> = &'a [u8];

fn parse_something(input: Input) -> nom::IResult<Input, ParsedType> {
}
```

**Rationale**: Slices are cheap to slice (just pointer+len), naturally track position, and work with nom parser combinators.

### 2. Parser Return Type

**Convention**: Use nom's `IResult` type for consistent error handling

```rust
use nom::IResult;

fn parse_u16(input: Input) -> IResult<Input, u16> {
    let (remaining, bytes) = nom::bytes::complete::take(2usize)(input)?;
    let value = u16::from_be_bytes(bytes.try_into().unwrap());
    Ok((remaining, value))
}
```

**Rationale**: Provides `(remaining_input, parsed_value)` tuple with automatic error propagation and location tracking.

### 3. Endianness Conventions

**Convention**: Explicitly specify endianness for multi-byte values

```rust
// Big-endian (network order, WASM uses this)
fn parse_u16_be(input: Input) -> IResult<Input, u16> {
    nom::number::complete::be_u16(input)
}

// Little-endian (x86, many formats)
fn parse_u16_le(input: Input) -> IResult<Input, u16> {
    nom::number::complete::le_u16(input)
}
```

**Rationale**: Different binary formats use different byte ordering; explicit markers prevent bugs.

### 4. Variable-Length Integer Encoding

**Convention**: LEB128 for compact integer encoding

```rust
fn parse_leb128_u32(input: Input) -> IResult<Input, u32> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;

    let mut remaining = input;
    loop {
        let (rest, byte) = nom::bytes::complete::take(1usize)(remaining)?;
        remaining = rest;

        let value = byte[0] & 0x7F;
        result |= (value as u32) << shift;

        if byte[0] & 0x80 == 0 {
            return Ok((remaining, result));
        }

        shift += 7;
        if shift >= 32 {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::TooLarge
            )));
        }
    }
}
```

**Rationale**: Variable-length encoding saves space for small numbers, standard in many binary formats (WASM, DWARF, Protocol Buffers).

### 5. Tag-Based Parsing

**Convention**: Use tag/prefix bytes to distinguish variants

```rust
#[derive(Debug)]
enum MyEnum {
    VariantA(u32),
    VariantB(String),
}

fn parse_my_enum(input: Input) -> IResult<Input, MyEnum> {
    let (remaining, tag) = nom::bytes::complete::take(1usize)(input)?;

    match tag[0] {
        0x01 => {
            let (rest, value) = parse_leb128_u32(remaining)?;
            Ok((rest, MyEnum::VariantA(value)))
        }
        0x02 => {
            let (rest, name) = parse_string(rest)?;
            Ok((rest, MyEnum::VariantB(name)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag
            )))
    }
}
```

**Rationale**: Enumerations in binary formats need type markers to distinguish variants during parsing.

### 6. Vector/List Parsing

**Convention**: Length-prefixed vectors with LEB128 count

```rust
fn parse_vector<T, P>(input: Input, parser: P) -> IResult<Input, Vec<T>>
where
    P: nom::Parser<Input, T, nom::error::Error<Input>>,
{
    let (remaining, count) = parse_leb128_u32(input)?;
    let (rest, items) = nom::multi::count(parser, count as usize)(remaining)?;
    Ok((rest, items))
}
```

**Usage**:
```rust
fn parse_u32_vector(input: Input) -> IResult<Input, Vec<u32>> {
    parse_vector(input, parse_leb128_u32)
}
```

**Rationale**: Vectors need size information to allocate correctly; LEB128 count makes format compact.

### 7. String Parsing

**Convention**: Length-prefixed UTF-8 strings

```rust
fn parse_string(input: Input) -> IResult<Input, String> {
    let (remaining, length) = parse_leb128_u32(input)?;
    let (rest, bytes) = nom::bytes::complete::take(length as usize)(remaining)?;

    match std::str::from_utf8(bytes) {
        Ok(s) => Ok((rest, s.to_string())),
        Err(_) => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify
            )))
    }
}
```

**Rationale**: Strings need length prefix and encoding validation (typically UTF-8).

### 8. Structured Data Parsing

**Convention**: Parse fields sequentially, using combinators

```rust
use nom::sequence::tuple;

struct MyStruct {
    field1: u32,
    field2: String,
    field3: Vec<u8>,
}

fn parse_my_struct(input: Input) -> IResult<Input, MyStruct> {
    let (remaining, (field1, field2, field3)) = tuple((
        parse_leb128_u32,
        parse_string,
        parse_vector(parse_byte)
    ))(input)?;

    Ok((remaining, MyStruct {
        field1,
        field2,
        field3,
    }))
}
```

**Rationale**: `tuple` combinator composes parsers sequentially; each parser advances position.

### 9. Error Handling with Location

**Convention**: Track position for accurate error messages

```rust
use nom::error_position;

fn parse_magic(input: Input) -> IResult<Input, [u8; 4]> {
    let (remaining, bytes) = nom::bytes::complete::tag(b"magic")(input)?;

    if bytes != [0x4D, 0x41, 0x47, 0x49] {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Tag,
        }));
    }

    Ok((remaining, bytes.try_into().unwrap()))
}

// Error at position 7 means byte offset 7 in original input
```

**Rationale**: Byte offsets provide exact location of parse failures for debugging.

### 10. Optional Fields

**Convention**: Use `opt` combinator for optional fields

```rust
use nom::combinator::opt;

fn parse_optional_field(input: Input) -> IResult<Input, Option<u32>> {
    let (remaining, value) = opt(parse_leb128_u32)(input)?;
    Ok((remaining, value))
}
```

**Rationale**: Binary formats often have optional fields that may be present or absent.

### 11. Validation After Parsing

**Convention**: Validate invariants after successful parse

```rust
fn parse_version(input: Input) -> IResult<Input, (u8, u8, u8, u8)> {
    let (remaining, bytes) = nom::bytes::complete::take(4usize)(input)?;
    let version = bytes;

    if version != [0x01, 0x00, 0x00, 0x00] {
        return Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Verify,
        }));
    }

    Ok((remaining, (version[0], version[1], version[2], version[3])))
}
```

**Rationale**: Post-parse validation catches semantic errors that parsing alone cannot detect.

## Testing Conventions

### 1. Round-Trip Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn test_roundtrip_encode_decode(value: u32) {
        let encoded = encode_leb128_u32(value);
        let (remaining, decoded) = parse_leb128_u32(&encoded).unwrap();
        assert_eq!(value, decoded);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_leb128_values() {
        test_roundtrip_encode_decode(0);
        test_roundtrip_encode_decode(127);
        test_roundtrip_encode_decode(128);
        test_roundtrip_encode_decode(65535);
        test_roundtrip_encode_decode(u32::MAX);
    }
}
```

### 2. Error Case Testing

```rust
#[test]
fn test_invalid_magic() {
    let input = [0x4D, 0x41, 0x47, 0x52]; // Wrong byte

    let result = parse_magic(&input);
    assert!(result.is_err());

    if let Err(nom::Err::Error(e)) = result {
        assert_eq!(e.code, nom::error::ErrorKind::Tag);
        assert_eq!(e.input.as_ptr(), input.as_ptr()); // Position points to error
    }
}
```

### 3. Property-Based Testing

```rust
#[test]
fn test_leb128_random_values() {
    for _ in 0..1000 {
        let value: u32 = rand::random();
        test_roundtrip_encode_decode(value);
    }
}
```

## Performance Conventions

1. **Avoid allocations** in hot path (use slices, not Vec where possible)
2. **Batch reads** - read multiple bytes at once when possible
3. **Early validation** - validate as soon as possible, fail fast
4. **Zero-copy parsing** - slice into input buffer, don't copy bytes

## Common Patterns

### Pattern 1: Length-Prefixed Data

```rust
fn parse_length_prefixed(input: Input) -> IResult<Input, Vec<u8>> {
    let (remaining, length) = parse_leb128_u32(input)?;
    let (rest, data) = nom::bytes::complete::take(length as usize)(remaining)?;
    Ok((rest, data.to_vec()))
}
```

### Pattern 2: Discriminated Union

```rust
fn parse_discriminated_union(input: Input) -> IResult<Input, Union> {
    let (remaining, discriminant) = nom::bytes::complete::take(1usize)(input)?;

    match discriminant[0] {
        0x00 => parse_variant_a(remaining),
        0x01 => parse_variant_b(remaining),
        0x02 => parse_variant_c(remaining),
        _ => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag
            )))
    }
}
```

### Pattern 3: Recursive Parsing

```rust
fn parse_tree(input: Input) -> IResult<Input, Tree> {
    let (remaining, tag) = nom::bytes::complete::take(1usize)(input)?;

    match tag[0] {
        0x00 => {
            let (rest, value) = parse_leb128_u32(remaining)?;
            Ok((rest, Tree::Leaf(value)))
        }
        0x01 => {
            let (rest, count) = parse_leb128_u32(remaining)?;
            let (rest2, children) = nom::multi::count(
                parse_tree,
                count as usize
            )(rest)?;
            Ok((rest2, Tree::Node(children)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag
            )))
    }
}
```

## nom Combinators to Use

- `tag` - Match exact byte sequence
- `take` - Take N bytes
- `be_u16`, `be_u32` - Parse big-endian numbers
- `le_u16`, `le_u32` - Parse little-endian numbers
- `tuple` - Compose parsers sequentially
- `alt` - Try alternatives, return first success
- `opt` - Make parser optional (returns Option)
- `many0`, `many1` - Parse 0 or more, 1 or more
- `count` - Parse exact count using given parser
- `all_consuming` - Ensure all input consumed

## Reference

- nom documentation: https://docs.rs/nom/latest/nom/
- LEB128: https://en.wikipedia.org/wiki/LEB128
