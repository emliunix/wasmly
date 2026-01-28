# WASM Binary Parser Implementation Plan

This document outlines the plan for implementing a WebAssembly binary parser for the wasmly.rs project.

## Goals

1. Parse .wasm binary files into structured Rust representations
2. Maintain accurate source location information for errors and debugging
3. Integrate with existing `Instr` and `Ty` types in the codebase
4. Support full WASM 1.0 binary format specification
5. Provide comprehensive testing

## Architecture Overview

```
wasm file → Parser → Module (types, functions, exports) → VM/Instance → Execution
```

### Key Design Decisions

1. **Use nom parser combinators** for robust, composable parsing
2. **Source location tracking** using `nom::error_position!` macro for accurate error reporting
3. **Slice-based input** - parse from `&[u8]` slices, track positions for location info
4. **Phased implementation** - build utilities → section parsers → instruction parser
5. **Type preservation** - map binary types directly to existing `Ty` enum

## Module Structure

### New Module: `src/binary/mod.rs`

```rust
pub struct Module {
    pub types: Vec<Ty>,
    pub functions: Vec<Func>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub start: Option<FuncIdx>,
}
```

### Section Structure: `src/binary/sections.rs`

Each section gets its own parser module:
- `parse_type_section()` - Type section (id 1)
- `parse_import_section()` - Import section (id 2)
- `parse_function_section()` - Function section (id 3)
- `parse_code_section()` - Code section (id 10) - most complex
- `parse_export_section()` - Export section (id 7)

## Implementation Phases

### Phase 1: LEB128 Utilities (`src/binary/leb128.rs`)

Purpose: Provide encoding/decoding utilities for variable-length integers

```rust
pub fn decode_u32(input: &[u8]) -> nom::IResult<&[u8], u32> { }
pub fn decode_i32(input: &[u8]) -> nom::IResult<&[u8], i32> { }
pub fn encode_u32(value: u32) -> Vec<u8> { }
pub fn encode_i32(value: i32) -> Vec<u8> { }
```

Test cases:
- Decode edge cases (0, 1, 127, 128, 65535)
- Test max values (u32::MAX, u64::MAX)
- Test sign extension for negative numbers
- Test continuation bit handling

### Phase 2: Binary Primitives (`src/binary/primitives.rs`)

Purpose: Parse basic WASM binary constructs

```rust
pub fn parse_magic(input: &[u8]) -> nom::IResult<&[u8], &[u8; 4]> { }
pub fn parse_version(input: &[u8]) -> nom::IResult<&[u8], [u8; 4]> { }
pub fn parse_section_header(input: &[u8]) -> nom::IResult<&[u8], (u8, u32)> { }
pub fn parse_name(input: &[u8]) -> nom::IResult<&[u8], String> { }
pub fn parse_vec<T, P>(input: &[u8], parser: P) -> nom::IResult<&[u8], Vec<T>> { }
```

Convention: Use specific error types with source location tracking

```rust
#[derive(Debug)]
pub struct ParseError {
    pub kind: nom::error::ErrorKind,
    pub position: usize,
}
```

### Phase 3: Section Parsers (`src/binary/sections.rs`)

Purpose: Parse each WASM section according to spec

```rust
pub fn parse_type_section(input: &[u8]) -> nom::IResult<&[u8], Vec<Ty>> { }
pub fn parse_import_section(input: &[u8]) -> nom::IResult<&[u8], Vec<Import>> { }
pub fn parse_function_section(input: &[u8]) -> nom::IResult<&[u8], Vec<TypeIdx>> { }
pub fn parse_code_section(input: &[u8]) -> nom::IResult<&[u8], Vec<CodeEntry>> { }
pub fn parse_export_section(input: &[u8]) -> nom::IResult<&[u8], Vec<Export>> { }
```

Convention: Each parser returns `(remaining_input, parsed_result)`

### Phase 4: Instruction Parser (`src/binary/instructions.rs`)

Purpose: Map binary instruction encodings to existing `Instr` enum

```rust
pub fn parse_instructions(input: &[u8]) -> nom::IResult<&[u8], Vec<Instr>> { }
pub fn parse_expr(input: &[u8]) -> nom::IResult<&[u8], Instr> { }
```

Key challenges:
- Map block type parsing (0x40, value type, or signed index)
- Handle variable-length immediates after each opcode
- Parse nested control structures (block/loop/if with end/else)
- Maintain correct instruction ordering

### Phase 5: Error Handling (`src/binary/error.rs`)

Purpose: Define error types and conversion utilities

```rust
pub enum BinaryError {
    InvalidMagic,
    InvalidVersion,
    UnknownSection,
    InvalidSectionSize,
    UnexpectedEOF,
    TypeMismatch,
    InvalidInstruction,
}
```

Convention: Use `nom::error_position!` for all parse errors:

```rust
use nom::error_position!(input, nom::error::ErrorKind::Tag);
```

### Phase 6: Main Parser (`src/binary/parser.rs`)

Purpose: Orchestrate parsing of complete WASM module

```rust
pub fn parse_module(input: &[u8]) -> Result<Module, BinaryError> { }
```

Convention: Return `Module` struct with all sections populated

## Binary Parsing Conventions

### LEB128 Encoding Rules

- Unsigned integers: Use continuation bit 0x80 for all bytes except last
- Signed integers: Use continuation bit 0x80, sign-extend to N bits where N = ceil(bit_size / 7)
- Size constraint: Total bytes ≤ ceil(N / 7) where N is integer size in bits
- Zero encoding: Multiple valid forms (0x00, 0x8000, 0x808000, etc.)

### Section Parsing Conventions

- Section header: `section_id:byte` + `length:u32` + `contents`
- Vectors: `count:u32` + `element^*` (vector of parsed elements)
- Custom sections: Can appear anywhere, skipped by default
- Section order: Custom sections can appear anywhere, others must follow spec order (1-13)
- Empty sections: Valid, equivalent to empty vector

### Instruction Parsing Conventions

- Opcodes: Single byte for simple instructions, multi-byte prefixes for complex
- Block type encoding:
  - `0x40` → Empty block type
  - `0x7F` → i32 value type
  - `0x7E` → i64 value type
  - `0x7D` → f32 value type
  - Positive signed s33 → Type index
- Structured instructions: `block`/`loop`/`if` + `type` + `instructions` + `0x0B` (end)
- `if` variants: Without else (then only), With else (then + `0x05` + `else` + `0x0B`)
- Immediate encoding: Variable-length LEB128 for indices, inline for small constants

### Error Reporting Conventions

- Use byte offsets (0-indexed from start of input)
- Include expected vs actual values
- Provide context (current section, expected construct)
- Reference the spec section that applies

### Testing Strategy

1. **Unit Tests**: Test each parsing function in isolation
2. **Integration Tests**: Parse complete .wasm files, verify all sections
3. **Round-trip Tests**: Encode → Parse → Verify matches original
4. **Fuzz Testing**: Use wabt test suite or simple fuzzer
5. **Invalid Input Tests**: Corrupted binaries, invalid magic, wrong versions

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
nom = "8"
```

## Implementation Order

1. Create module structure with placeholder types
2. Implement LEB128 utilities with tests
3. Implement binary primitives (magic, version, sections)
4. Implement section parsers (one at a time)
5. Implement instruction parser (largest piece)
6. Integrate with existing VM/Instance
7. Add integration tests

## Success Criteria

- Parser can read all .wasm files in wabt test suite
- Parser produces same `Instr` sequences as existing code
- Error messages include accurate source location (byte offset)
- All tests pass with `cargo test`
- Code follows AGENTS.md style guidelines
- No clippy warnings

## Reference Materials

- WASM Spec: `./wasm-spec/document/core/binary/`
- nom documentation: https://docs.rs/nom/latest/nom/
- LEB128 Wikipedia: https://en.wikipedia.org/wiki/LEB128
- wabt repository: https://github.com/WebAssembly/wabt
