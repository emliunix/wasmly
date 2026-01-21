# wasmly - Agent Guidelines

This is a WebAssembly interpreter focused on durable and suspendable execution. All agentic coding agents should follow these guidelines.

## Build Commands

### Core Commands
```bash
# Build the project
cargo build

# Build with optimizations
cargo build --release

# Run the binary
cargo run

# Run all tests
cargo test

# Run a single test (replace with actual test name)
cargo test test_name
cargo test -- test_name

# Run tests with output
cargo test -- --nocapture

# Run tests from a specific module
cargo test mod_name::tests

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run linter (clippy)
cargo clippy

# Fix clippy warnings automatically
cargo clippy --fix

# Build and run clippy with strict warnings
cargo clippy -- -D warnings
```

### Current Test Status
- Total tests: 6
- Passing: 5
- Failing: 1 (tests::test_loop - overflow bug)
- Test modules: `src/main.rs` (tests), `src/cont.rs` (tests)

## Code Style Guidelines

### Imports and Modules
- Module declarations at top: `mod types; mod cont;`
- Wildcard imports for common types: `use types::*;`
- External imports listed first, then local wildcard imports
- Group related imports together

### Formatting
- Uses default rustfmt (no custom .rustfmt.toml)
- Always run `cargo fmt` before committing
- Max line length: 100 characters (default rustfmt)
- 4-space indentation for blocks
- Align function arguments where readable

### Types and Type Definitions
- **Enums**: PascalCase (e.g., `Ty`, `Instr`, `Val`, `BlockType`)
- **Structs**: PascalCase (e.g., `VM`, `Instance`, `Level`, `Config`)
- **Traits**: PascalCase (e.g., `InstrCursor`)
- **Type aliases**: PascalCase
- Use `#[derive(Debug, Clone)]` for types that are copied
- Use `PartialEq, Eq` for value equality comparison

### Naming Conventions
- **Functions/Methods**: snake_case (e.g., `new`, `run`, `step`, `push`, `pop`, `num_rets`)
- **Variables**: snake_case (e.g., `instrs`, `cursor`, `locals`, `types`)
- **Constants**: SCREAMING_SNAKE_CASE (not currently used)
- **Fields**: snake_case (e.g., `cur`, `len`, `halt`)
- **Lifetimes**: short names like `'a`, `'b`
- **Macros**: snake_case (e.g., `impl_stack_push`, `impl_val`)

### Visibility
- Public API: `pub` keyword
- Internal implementations: private by default
- Test helpers: private, in `#[cfg(test)]` modules

### Error Handling
- Current approach: heavy use of `panic!()` and `unwrap()` for unreachable paths
- Common patterns:
  - `panic!("unreachable")` - for logically unreachable code
  - `panic!("impossible")` - for invariant violations
  - `panic!("type mismatch")` - for type errors
  - `.unwrap()` - for values that should never be None/Err
- **Future improvement**: Consider using `Result<T, E>` for recoverable errors
- Always handle potential panics in production code paths

### Macros
- Used for code reduction in repetitive patterns
- Macro naming: snake_case with descriptive suffix
  - `impl_stack_push!` - generates push methods
  - `impl_stack_pop!` - generates pop methods
  - `impl_val!` - generates value extraction functions
  - `stack_val!` - matches stack items
- Keep macros simple and well-documented

### Functions and Methods
- **Constructors**: `fn new() -> Self`
- **Main execution**: `fn run(&mut self, ...)`
- **Step execution**: `fn step(&mut self, ...)`
- **Stack operations**: `fn push`, `fn pop`
- **Inline helpers**: Use `#[inline]` for small, frequently called functions
- Avoid unused variables; prefix with underscore if intentionally unused

### Testing
- Tests in `#[cfg(test)] mod tests` at end of each file
- Test naming: `test_<feature>` (e.g., `test`, `test_block`, `test_loop`)
- Arrange-Act-Assert pattern preferred
- Use `assert_eq!` and `assert!` for assertions
- Keep tests focused on single behaviors

### Pattern Matching
- Exhaustive matching preferred
- Use `_` for unreachable patterns with `panic!()`
- Destructure enums directly in match arms
- Use `if let` for simple single-case matches

### Generic Constraints
- Use trait bounds explicitly (e.g., `<T: Clone>`, `<I: Index<usize, Output=Ty>>`)
- Prefer lifetime bounds over clones where possible
- Document why generic constraints are needed

### Code Organization
- **src/types.rs**: Core type definitions (Ty, Instr, Val, BlockType)
- **src/main.rs**: VM implementation using cursor-based approach
- **src/cont.rs**: Continuation-based interpreter implementation (alternative approach)
- Each module is self-contained with its own tests

### Documentation
- Public APIs should have doc comments (`///`)
- Complex algorithms need inline comments
- Use Rust's `#[doc(...)]` attributes for advanced docs
- Keep comments up-to-date with code changes

### Dependencies
- **wasmparser** - WebAssembly binary format parsing
- **wast** - WebAssembly text format parsing
- **bumpalo** - Bump allocator
- **leb128fmt** - LEB128 encoding/decoding
- Add new dependencies to Cargo.toml with version pinning

### WebAssembly Spec Reference
- **IMPORTANT**: Only read WebAssembly spec from `./wasm-spec/` path
- Do NOT follow symlink, do NOT search web
- **Always consult `docs/wasm-spec.md` first** for file listings and spec structure
- Use `docs/wasm-spec.md` to locate spec files instead of using glob/grep searches
- Reference for implementing instructions: `wasm-spec/document/core/exec/instructions.rst`
- Reference for types: `wasm-spec/document/core/syntax/modules.rst`
- Reference for validation: `wasm-spec/document/core/valid/instructions.rst`
- Test files at `test/core/*.wast` serve as behavioral reference

### Known Issues
- Test `tests::test_loop` in `src/main.rs` has an overflow bug (line 291)
- Clippy warnings for unused variables and dead code (use `_` prefix for intentional unused)
- Two interpreter implementations (VM and Instance) - consolidate in future

<<<<<<< HEAD
### Before Committing
1. Run `cargo fmt` to format code
2. Run `cargo test` to ensure all tests pass
3. Run `cargo clippy -- -D warnings` to check for issues
4. Review changes for unused code and variables
5. Ensure tests cover new functionality
=======
**Completed:**
- **LEB128 utilities** (`src/binary/leb128.rs`)
  - `decode_u32()`, `decode_i32()` - Decode unsigned/signed 32-bit integers
  - `encode_u32()`, `encode_i32()` - Encode integers to bytes
  - Comprehensive test coverage for edge cases (0, 127, 128, max values, negative numbers)

- **Binary primitives** (`src/binary/primitives.rs`) - ✅ ALL TESTS PASSING
  - `parse_byte()` - Single byte parser with location tracking
  - `parse_magic()` - Magic number validation (`\0asm`)
  - `parse_version()` - Version validation (`1`)
  - `parse_section_header()` - Parse section id and length with proper offset tracking
  - `parse_name()` - Parse length-prefixed UTF-8 strings
  - `parse_leb128_u32()` - Parse LEB128-encoded u32 values
  - All 8 tests passing: byte, magic, version, leb128 (small/medium/large), name, section header
  - Location tracking correctly reports offset (starting position) and length (bytes consumed)

- **Error types** (`src/binary/error.rs`)
  - `BinaryError` enum for common parse errors
  - `ParseResult` type alias for nom IResult with source location tracking
  - `Located<T>` wrapper providing source location for parsed values
  - `SourceLocation` struct tracking offset and length in input stream

- **Module data model** (`src/module.rs` and `src/types.rs`)
  - Complete type system following WASM spec
  - All 12 section structures (Type, Import, Function, Table, Memory, Global, Export, Start, Element, Code, Data, DataCount, Custom)
  - Runtime structures (Store, ModuleInst, FuncInst, TableInst, MemInst, GlobalInst, ElemInst, DataInst)
  - Embedder API skeleton with `todo!()` placeholders for future implementation
  - Index types for type safety (TypeIdx, FuncIdx, TableIdx, MemIdx, GlobalIdx, etc.)

**Documentation:**
- `docs/binary-parsing-conventions.md` - General binary parsing patterns and conventions
  - Input representation with byte slices
  - Variable-length integer encoding (LEB128)
  - Tag-based parsing, vectors, strings
  - Error handling with location tracking
  - Testing conventions and common patterns

- `docs/wasm-spec-tests.md` - WebAssembly specification test suite guide
  - Overview of 90+ test files in wasm-spec/test/core/
  - Test file format (.wast) explanation
  - Simple test cases with binary breakdowns
  - Test organization by development phase
  - Methods for extracting and using test cases
  - Recommended test harness structure

**Remaining:**
- Section parsers (type, import, function, export, data, code sections)
- Instruction parser mapping binary opcodes to `Instr` enum
- Module-level parser orchestrating all sections
- Integration with existing VM/Instance interpreters
- Implementation of embedder API functions (module_decode, module_instantiate, func_invoke, etc.)

### Module Structure

The project follows a two-layer architecture for WASM modules:

1. **Compile-time representation** (`Module` struct):
   - Contains all 12 section types as defined in the WASM spec
   - Sections: types, imports, functions, tables, memories, globals, exports, start, elements, code, data, data_count, customs
   - Methods: `module_imports()`, `module_exports()`, `validate()` (to be implemented)

2. **Runtime representation** (`ModuleInst` struct):
   - Contains addresses (indices) into the Store for all runtime instances
   - Represents an instantiated module with resolved imports

3. **Store** (`Store` struct):
   - Global runtime state holding all instances
   - Contains vectors of: funcs, tables, mems, globals, elems, datas

**Index Spaces**: 
- Imports appear FIRST in each index space, followed by module-defined items
- Example: Function index 0-n are imported functions, n+1 onwards are module functions
- This is critical for correct resolution of indices during instantiation and execution
>>>>>>> 6dbf3d6 (fix: resolve test compilation errors in binary parser primitives)
