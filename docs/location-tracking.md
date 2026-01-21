# Location Tracking in Binary Parser

## Overview

The binary parser uses explicit location tracking to enable snapshot execution and precise error reporting. All parsed values are wrapped in `Located<T>` with their source location information.

## Key Components

### `SourceLocation`

Represents a range of bytes in the original binary file:

```rust
pub struct SourceLocation {
    pub offset: usize,    // Starting byte offset
    pub length: usize,    // Length in bytes
}
```

**Methods:**
- `new(offset, length)` - Create new location
- `from_slice(base, slice)` - Create from base slice and current slice
- `end()` - Get ending offset
- `contains(offset)` - Check if byte offset is within range

### `Located<T>`

Wraps a parsed value with its source location:

```rust
pub struct Located<T> {
    pub value: T,              // The parsed value
    pub location: SourceLocation, // Where it came from
}
```

**Methods:**
- `new(value, location)` - Create new located value
- `from_parse(base, value, remaining)` - Create from parse result
- `map(f)` - Transform value, preserving location
- `location()` - Get location reference
- `value()` - Get value reference
- `into_inner()` - Extract value

## Usage Patterns

### 1. Location-Aware Parsers

All parsers return `Located<T>` instead of bare `T`:

```rust
pub fn parse_byte(input: Input) -> ParseResult<'_, Located<u8>> {
    let base = input;
    let (remaining, bytes) = take(1usize)(input)?;
    let offset = base.len() - remaining.len();
    let location = SourceLocation::new(offset, 1);
    Ok((remaining, Located::new(bytes[0], location)))
}
```

### 2. Accessing Parsed Values

```rust
let (remaining, byte) = parse_byte(&input)?;
println!("Byte {} at offset {}", byte.value, byte.location.offset);
```

### 3. Transforming Values While Preserving Location

```rust
let (remaining, count) = parse_leb128_u32(&input)?;
let length: usize = count.map(|v| v as usize).into_inner();
```

## Benefits for Snapshot Execution

1. **Error Mapping** - Map execution errors back to original binary locations
2. **Debugging** - Know exactly where each instruction/function is defined
3. **Code Coverage** - Track which parts of binary were executed
4. **Profiling** - Measure execution time per instruction with source info
5. **Verification** - Compare parsed structure with original binary

## Implementation Notes

### Offset Calculation

Offset is calculated by comparing input slices:

```rust
let offset = base.len() - remaining.len();
```

This works because:
- `base` is the original input slice
- `remaining` is the slice after parsing
- The difference is bytes consumed = offset

### Zero-Copy

Location tracking is zero-copy:
- No allocation for location calculation
- Uses slice pointer arithmetic
- Minimal overhead

### Error Position

Nom's error types include input position automatically:

```rust
return Err(nom::Err::Error(nom::error::Error {
    input: base,  // Points to error location
    code: nom::error::ErrorKind::Tag,
}));
```

## Testing

All location-aware parsers have tests verifying:
1. Correct parsed value
2. Correct offset calculation
3. Correct length
4. Remaining input matches expected

Example test:

```rust
#[test]
fn test_parse_byte_location() {
    let input = [0x42, 0xFF];
    let (remaining, byte) = parse_byte(&input).unwrap();
    assert_eq!(byte.value, 0x42);
    assert_eq!(byte.location.offset, 0);
    assert_eq!(byte.location.length, 1);
    assert_eq!(remaining, [0xFF][..]);
}
```

## Future Enhancements

1. **Module-level offset tracking** - Track absolute offsets from file start
2. **Location-aware error types** - Include location in error messages
3. **Source maps** - Map binary locations to original source (if available)
4. **Pretty printing** - Display locations in hex format (0x1A, 0x2B, etc.)
