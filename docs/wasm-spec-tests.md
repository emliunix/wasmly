# WebAssembly Spec Test Suite Guide

This document describes the WebAssembly specification test suite and how to leverage it for testing our binary parser implementation.

## Overview

The wasm-spec repository includes a comprehensive test suite with **90+ test files** in `.wast` format, specifically designed for testing WebAssembly implementations. These tests are production-grade and used by all major WASM implementations.

## Test File Location

**Primary Test Directory:** `wasm-spec/test/core/`

**Key Binary Parsing Test Files:**

| File | Size | Focus Area | Priority |
|------|------|-----------|----------|
| `binary.wast` | 47KB | General binary format | **High** |
| `binary-leb128.wast` | 43KB | LEB128 encoding edge cases | Medium |
| `custom.wast` | 3.6KB | Custom sections | **High** |
| `nop.wast` | 10KB | Simple instruction tests | Medium |
| `type.wast` | 1.4KB | Type section | **High** |
| `imports.wast` | 8.6KB | Import section | Medium |
| `exports.wast` | 7.4KB | Export section | Medium |
| `start.wast` | 1.9KB | Start section | Low |
| `global.wast` | 18KB | Global section | Medium |

**Total:** 90 .wast test files covering all aspects of WASM

## Test File Format (.wast)

The `.wast` format is an **S-expression based scripting language** for WASM testing.

### Structure

```scheme
;; Valid minimal module (8 bytes: magic + version)
(module binary "\00asm\01\00\00\00")

;; Module with sections (can be split across multiple strings)
(module binary 
  "\00asm" "\01\00\00\00"     ;; magic + version
  "\01\04\01\60\00\00"        ;; Type section: 1 type
  "\03\02\01\00"              ;; Function section: 1 function
)

;; Named module
(module $M1 binary "\00asm\01\00\00\00")

;; Test assertions - malformed binary
(assert_malformed 
  (module binary "") 
  "unexpected end"
)

;; Test assertions - validation failure
(assert_invalid 
  (module binary "\00asm\01\00\00\00" "\03\02\01\00")  ;; func without type
  "unknown type"
)

;; Test assertions - execution result
(assert_return (invoke "foo") (i32.const 42))

;; Test assertions - runtime trap
(assert_trap (invoke "divide_by_zero") "integer divide by zero")
```

### Key Elements

1. **`(module binary <hex-string>*)`** - Define binary modules as hex escape sequences
2. **`(assert_malformed <module> <error-msg>)`** - Test invalid binary format
3. **`(assert_invalid <module> <error-msg>)`** - Test validation failures
4. **`(assert_return <action> <result>*)`** - Test execution results
5. **`(assert_trap <action> <error-msg>)`** - Test runtime traps
6. **`(invoke <name> <args>*)`** - Call exported functions

### Hex String Format

Hex strings in .wast files use escape sequences:
- `\00` - Null byte (0x00)
- `\01` - Byte 0x01
- `\7f` - Byte 0x7F (127)
- `\ff` - Byte 0xFF (255)
- Regular text is UTF-8 encoded: `"abc"` → `0x61 0x62 0x63`

## Simple Test Cases

### Test Case 1: Minimal Valid Module (8 bytes)

**Source:** `binary.wast:1`

```scheme
(module binary "\00asm\01\00\00\00")
```

**Binary (hex):**
```
00 61 73 6d 01 00 00 00
```

**Breakdown:**
- `00 61 73 6d` - Magic number "\0asm"
- `01 00 00 00` - Version 1 (little-endian)

**Expected:** Parse successfully, empty module with no sections

### Test Case 2: Invalid Magic Numbers

**Source:** `binary.wast:6-18`

```scheme
;; Truncated magic
(assert_malformed (module binary "") "unexpected end")
(assert_malformed (module binary "\01") "unexpected end")
(assert_malformed (module binary "\00as") "unexpected end")

;; Wrong magic
(assert_malformed (module binary "asm\00") "magic header not detected")
(assert_malformed (module binary "wasm\01\00\00\00") "magic header not detected")
(assert_malformed (module binary "\00ASM\01\00\00\00") "magic header not detected")
```

**Expected:** All should fail with appropriate error messages

### Test Case 3: Invalid Version Numbers

**Source:** `binary.wast:37-45`

```scheme
;; Truncated version
(assert_malformed (module binary "\00asm") "unexpected end")
(assert_malformed (module binary "\00asm\01") "unexpected end")

;; Wrong version
(assert_malformed (module binary "\00asm\00\00\00\00") "unknown binary version")
(assert_malformed (module binary "\00asm\0d\00\00\00") "unknown binary version")
(assert_malformed (module binary "\00asm\0e\00\00\00") "unknown binary version")
(assert_malformed (module binary "\00asm\00\01\00\00") "unknown binary version")
(assert_malformed (module binary "\00asm\00\00\01\00") "unknown binary version")
```

**Expected:** All should fail, only version `01 00 00 00` is valid

### Test Case 4: Custom Sections

**Source:** `custom.wast:1-12`

```scheme
(module binary
  "\00asm" "\01\00\00\00"
  "\00\24\10" "a custom section" "this is the payload"
  "\00\20\10" "a custom section" "this is payload"
  "\00\11\10" "a custom section" ""
  "\00\10\00" "" "this is payload"
  "\00\01\00" "" ""
)
```

**Binary breakdown for first custom section:**
- `\00` - Section ID 0 (custom section)
- `\24` - Section size (36 bytes)
- `\10` - Name length (16 bytes)
- `"a custom section"` - Name (16 bytes UTF-8)
- `"this is the payload"` - Payload (19 bytes)

**Expected:** Parse successfully with 5 custom sections

### Test Case 5: Type Section

**Source:** `binary.wast` (various lines)

```scheme
(module binary
  "\00asm" "\01\00\00\00"
  "\01\05\01\60\00\01\7f"       ;; Type section
)
```

**Binary breakdown:**
- `\01` - Section ID 1 (type section)
- `\05` - Section size (5 bytes)
- `\01` - Number of types (1)
- `\60` - Function type tag
- `\00` - Number of parameters (0)
- `\01` - Number of results (1)
- `\7f` - Result type i32

**Expected:** Module with 1 function type: `[] -> [i32]`

### Test Case 6: Function Declaration with Code

**Source:** `binary.wast:54-73`

```scheme
(module binary
  "\00asm" "\01\00\00\00"
  "\01\04\01\60\00\00"       ;; Type section: func [] -> []
  "\03\02\01\00"             ;; Function section: 1 func, type 0
  "\0a\04\01\02\00\0b"       ;; Code section: 1 func body
)
```

**Binary breakdown:**

Type section:
- `\01` - Section ID 1
- `\04` - Section size (4 bytes)
- `\01` - 1 type
- `\60\00\00` - func type: [] -> []

Function section:
- `\03` - Section ID 3
- `\02` - Section size (2 bytes)
- `\01` - 1 function
- `\00` - Function 0 uses type 0

Code section:
- `\0a` - Section ID 10
- `\04` - Section size (4 bytes)
- `\01` - 1 code entry
- `\02` - Code size (2 bytes)
- `\00` - 0 locals
- `\0b` - end instruction

**Expected:** Module with 1 function of type 0, empty body

### Test Case 7: LEB128 Edge Cases

**Source:** `binary-leb128.wast:2-6`

```scheme
;; Non-minimal LEB128 encoding (allowed by spec)
(module binary
  "\00asm" "\01\00\00\00"
  "\05\04\01"                ;; Memory section with 1 entry
  "\00\82\00"                ;; no max, minimum 2 (encoded as \82\00 instead of \02)
)
```

**LEB128 explanation:**
- `\02` - Minimal encoding of 2 (1 byte)
- `\82\00` - Non-minimal encoding of 2 (2 bytes)
  - Byte 1: `\82` = `10000010` → value bits `0000010`, continue bit set
  - Byte 2: `\00` = `00000000` → value bits `0000000`, stop
  - Result: `0000000 0000010` = 2

**Expected:** Parse successfully, non-minimal encodings are allowed

## Test Organization by Development Phase

### Phase 1: Module Header (Start Here)

**Files:** `binary.wast` (lines 1-50)

**Test cases:**
1. Valid magic + version (8 bytes)
2. Invalid magic variations
3. Invalid version variations
4. Truncated header tests

**Goal:** Parse and validate module header

### Phase 2: Section Iteration

**Files:** `custom.wast` (all), `binary.wast` (section structure tests)

**Test cases:**
1. Empty module (no sections)
2. Module with custom sections only
3. Multiple custom sections
4. Section size validation

**Goal:** Iterate through sections, handle custom sections

### Phase 3: Type Section

**Files:** `type.wast`, `binary.wast` (type section examples)

**Test cases:**
1. Simple function types: `[] -> []`, `[i32] -> []`, `[] -> [i32]`
2. Multiple parameter types: `[i32, i64] -> []`
3. Multiple result types: `[] -> [i32, i32]`
4. Complex types: `[i32, i64, f32, f64] -> [i32, i64]`

**Goal:** Parse type section, build function type vector

### Phase 4: Function + Code Sections

**Files:** `binary.wast` (function examples), `nop.wast`

**Test cases:**
1. Function section (type indices)
2. Code section (locals + instructions)
3. Function/Code section length matching
4. Simple instructions: nop, i32.const, drop

**Goal:** Parse function declarations and bodies

### Phase 5: Other Sections

**Files:** Section-specific test files

**Test cases:**
1. Import section (`imports.wast`)
2. Export section (`exports.wast`)
3. Memory section (`memory*.wast`)
4. Table section (`table*.wast`)
5. Global section (`global.wast`)
6. Element section (`elem.wast`)
7. Data section (`data.wast`)
8. Start section (`start.wast`)

**Goal:** Complete module parsing

### Phase 6: Complex Instructions & Edge Cases

**Files:** Instruction-specific tests, `binary-leb128.wast`

**Test cases:**
1. Control flow: block, loop, if, br
2. All arithmetic/logic instructions
3. LEB128 edge cases
4. Validation rules

**Goal:** Full WASM binary parser

## How to Use the Test Suite

### Method 1: Extract Binary Modules Manually

Create Rust test helpers to convert hex strings:

```rust
/// Convert WASM hex escape sequences to bytes
fn wast_hex_to_bytes(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut chars = hex.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                // Hex escape: \xx
                let h1 = chars.next().unwrap();
                let h2 = chars.next().unwrap();
                let byte = u8::from_str_radix(&format!("{}{}", h1, h2), 16).unwrap();
                bytes.push(byte);
            }
            _ => {
                // Regular UTF-8 character
                bytes.push(c as u8);
            }
        }
    }
    
    bytes
}

#[test]
fn test_minimal_module() {
    let bytes = wast_hex_to_bytes(r"\00asm\01\00\00\00");
    let module = parse_module(&bytes).unwrap();
    assert_eq!(module.version, 1);
    assert!(module.types.is_empty());
}
```

### Method 2: Direct Binary Embedding

For simple tests, embed binary directly in Rust:

```rust
#[test]
fn test_magic_version() {
    const MINIMAL: &[u8] = b"\x00asm\x01\x00\x00\x00";
    let result = parse_module(MINIMAL);
    assert!(result.is_ok());
}

#[test]
fn test_bad_magic() {
    const BAD: &[u8] = b"wasm\x01\x00\x00\x00";
    let result = parse_module(BAD);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("magic"));
}
```

### Method 3: Create Test Data Files

Create a `tests/data/` directory with .wasm files:

```
tests/
  data/
    minimal.wasm              (8 bytes: magic + version)
    bad_magic.wasm            (invalid magic)
    bad_version.wasm          (invalid version)
    custom_section.wasm       (with custom section)
    type_section.wasm         (with type section)
    function_simple.wasm      (function + code sections)
```

Load in tests:

```rust
#[test]
fn test_minimal_module() {
    let bytes = include_bytes!("data/minimal.wasm");
    let module = parse_module(bytes).unwrap();
    assert_eq!(module.types.len(), 0);
}

#[test]
fn test_type_section() {
    let bytes = include_bytes!("data/type_section.wasm");
    let module = parse_module(bytes).unwrap();
    assert_eq!(module.types.len(), 1);
    assert_eq!(module.types[0].params.len(), 0);
    assert_eq!(module.types[0].results.len(), 1);
}
```

### Method 4: Use wasm-spec Interpreter (Advanced)

The wasm-spec includes a reference interpreter that can convert .wast to .wasm:

```bash
cd wasm-spec/interpreter
make

# Convert .wat text format to .wasm binary
./wasm module.wat -o module.wasm

# Run a test script (validates all assertions)
./wasm test.wast
```

## Test Harness Structure

Recommended structure for parser tests:

```rust
// tests/binary_parser_tests.rs

mod helpers {
    /// Convert WASM hex escape sequences to bytes
    pub fn wast_hex_to_bytes(hex: &str) -> Vec<u8> {
        // Implementation above
    }
}

mod magic_version_tests {
    use super::helpers::*;
    use wasmly::binary::parse_module;
    
    #[test]
    fn test_minimal_valid() {
        let bytes = wast_hex_to_bytes(r"\00asm\01\00\00\00");
        assert!(parse_module(&bytes).is_ok());
    }
    
    #[test]
    fn test_bad_magic_wasm() {
        let bytes = b"wasm\x01\x00\x00\x00";
        let err = parse_module(bytes).unwrap_err();
        assert!(err.to_string().contains("magic"));
    }
    
    #[test]
    fn test_truncated_magic() {
        let bytes = b"\x00as";
        let err = parse_module(bytes).unwrap_err();
        assert!(err.to_string().contains("unexpected end"));
    }
}

mod custom_section_tests {
    use super::helpers::*;
    use wasmly::binary::parse_module;
    
    #[test]
    fn test_single_custom_section() {
        let bytes = wast_hex_to_bytes(
            r"\00asm\01\00\00\00\00\0e\06custom\70ayload"
        );
        let module = parse_module(&bytes).unwrap();
        assert_eq!(module.customs.len(), 1);
        assert_eq!(module.customs[0].name, "custom");
    }
}

mod type_section_tests {
    use super::helpers::*;
    use wasmly::binary::parse_module;
    
    #[test]
    fn test_empty_func_type() {
        let bytes = wast_hex_to_bytes(
            r"\00asm\01\00\00\00\01\04\01\60\00\00"
        );
        let module = parse_module(&bytes).unwrap();
        assert_eq!(module.types.len(), 1);
        assert_eq!(module.types[0].params.len(), 0);
        assert_eq!(module.types[0].results.len(), 0);
    }
}
```

## Additional Resources

### In wasm-spec
- **Binary format spec:** `wasm-spec/document/core/binary/`
- **Test format docs:** `wasm-spec/test/README.md`
- **Core tests README:** `wasm-spec/test/core/README.md`
- **Interpreter README:** `wasm-spec/interpreter/README.md`
- **Reference decoder:** `wasm-spec/interpreter/binary/decode.ml` (OCaml implementation)

### In Our Project
- **Parser plan:** `docs/parser.md`
- **Binary conventions:** `docs/binary-parsing-conventions.md`
- **Module data model:** `src/module.rs`
- **Type system:** `src/types.rs`
- **LEB128 utilities:** `src/binary/leb128.rs`

### External Tools
- **wabt toolkit** - `wasm2wat` / `wat2wasm` for binary ↔ text conversion
- **wasm-objdump** - Disassemble WASM binaries
- **hexdump** - Inspect binary files: `hexdump -C module.wasm`

## Summary

The wasm-spec test suite provides:
- ✅ **90+ comprehensive test files** covering all WASM aspects
- ✅ **Binary format tests** with inline hex-encoded modules
- ✅ **Positive and negative test cases** (valid and malformed)
- ✅ **Well-documented format** with comments explaining each byte
- ✅ **Production-grade quality** used by all major implementations

**Recommended development path:**
1. Start with `binary.wast` (lines 1-50) - magic/version tests
2. Add `custom.wast` (all) - simplest section type
3. Add `type.wast` - foundational type system
4. Progress through section-specific tests
5. Complete with instruction and edge case tests

This incremental approach ensures each parser component is thoroughly tested against the official specification before moving to the next component.
