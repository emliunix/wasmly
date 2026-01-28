# WebAssembly Specification - File Listings

This document provides a comprehensive listing of the WebAssembly specification repository structure for quick reference.

## Repository Structure

### Root Files
- **README.md** - Main repository documentation
- **Contributing.md** - Contribution guidelines
- **LICENSE** - License file
- **wasm-specs.bib** - Bibliography for citing WebAssembly
- **w3c.json** - W3C configuration

## Core Specification (`document/core/`)

### Syntax Definitions (`syntax/`)
- **modules.rst** - Module structure (types, functions, globals, memories, tables, data/element segments)
- **types.rst** - Type definitions (recursive types, subtypes)
- **values.rst** - Value representations
- **instructions.rst** - Instruction syntax
- **conventions.rst** - Notation conventions
- **index.rst** - Syntax index

### Execution Semantics (`exec/`)
- **runtime.rst** - Runtime structure (store, values, addresses, module/function/global/table/memory/instances)
- **modules.rst** - Module instantiation, allocation, invocation
- **instructions.rst** - Instruction execution semantics
- **numerics.rst** - Numeric operations
- **values.rst** - Value operations
- **types.rst** - Type operations
- **conventions.rst** - Execution conventions
- **index.rst** - Execution index

### Validation Rules (`valid/`)
- **modules.rst** - Module validation
- **instructions.rst** - Instruction validation (includes constant expressions)
- **types.rst** - Type validation
- **matching.rst** - Type matching rules
- **conventions.rst** - Validation conventions
- **index.rst** - Validation index

### Binary Format (`binary/`)
- **modules.rst** - Binary module encoding
- **types.rst** - Binary type encoding
- **instructions.rst** - Binary instruction encoding
- **values.rst** - Binary value encoding
- **conventions.rst** - Binary format conventions
- **index.rst** - Binary index

### Text Format (`text/`)
- **modules.rst** - Text format modules
- **types.rst** - Text format types
- **instructions.rst** - Text format instructions
- **values.rst** - Text format values
- **lexical.rst** - Lexical grammar
- **conventions.rst** - Text format conventions
- **index.rst** - Text index

### Introduction (`intro/`)
- **introduction.rst** - Overview
- **overview.rst** - High-level concepts
- **index.rst** - Introduction index

### Appendix (`appendix/`)
- **algorithm.rst** - Algorithms
- **properties.rst** - Mathematical properties
- **implementation.rst** - Implementation considerations
- **embedding.rst** - Embedder interface
- **changes.rst** - Version changes
- **custom.rst** - Custom sections
- **index-rules.rst** - Index rules
- **index-types.rst** - Index types
- **profiles.rst** - Profiles
- **index.rst** - Appendix index

### Configuration Files
- **conf.py** - Sphinx configuration
- **index.rst** - Main index
- **index.bs** - Bikeshed index
- **Makefile** - Build configuration

## Test Suite (`test/`)

### Core Tests (`test/core/`)

#### Basic Tests
- **address.wast** - Address tests
- **align.wast** - Alignment tests
- **annotations.wast** - Annotation tests
- **binary.wast** - Binary encoding tests
- **binary-leb128.wast** - LEB128 encoding tests
- **block.wast** - Block instruction tests
- **br.wast**, **br_if.wast**, **br_table.wast** - Branch instruction tests
- **call.wast**, **call_indirect.wast**, **call_ref.wast** - Call instruction tests
- **comments.wast** - Comment tests
- **const.wast** - Constant tests
- **conversions.wast** - Type conversion tests
- **custom.wast** - Custom section tests
- **data.wast** - Data segment tests
- **elem.wast** - Element segment tests
- **exports.wast** - Export tests
- **forward.wast** - Forward reference tests
- **func.wast**, **func_ptrs.wast** - Function tests
- **global.wast** - Global variable tests
- **i32.wast**, **i64.wast** - Integer operation tests
- **if.wast** - If instruction tests
- **imports.wast** - Import tests
- **instance.wast** - Instance tests
- **labels.wast** - Label tests
- **left-to-right.wast** - Evaluation order tests
- **linking.wast** - Linking tests
- **load.wast**, **store.wast** - Memory load/store tests
- **local_get.wast**, **local_set.wast**, **local_tee.wast** - Local variable tests
- **local_init.wast** - Local initialization tests
- **loop.wast** - Loop instruction tests
- **memory.wast**, **memory_grow.wast**, **memory_size.wast** - Memory tests
- **memory_trap.wast** - Memory trap tests
- **memory_redundancy.wast** - Memory redundancy tests
- **names.wast** - Name section tests
- **nop.wast** - No-op instruction tests
- **ref.wast**, **ref_null.wast**, **ref_is_null.wast**, **ref_func.wast** - Reference tests
- **return.wast**, **return_call.wast**, **return_call_indirect.wast**, **return_call_ref.wast** - Return tests
- **select.wast** - Select instruction tests
- **stack.wast** - Stack tests
- **start.wast** - Start function tests
- **store.wast** - Store instruction tests
- **switch.wast** - Switch tests
- **table.wast**, **table_get.wast**, **table_set.wast**, **table_grow.wast**, **table_size.wast** - Table tests
- **traps.wast** - Trap tests
- **type.wast**, **type-canon.wast**, **type-equivalence.wast**, **type-rec.wast** - Type tests
- **unreachable.wast** - Unreachable instruction tests
- **unreached-valid.wast**, **unreached-invalid.wast** - Unreached code tests
- **unwind.wast** - Unwind tests
- **utf8-import-field.wast**, **utf8-import-module.wast**, **utf8-invalid-encoding.wast** - UTF-8 tests

#### Floating Point Tests
- **f32.wast**, **f32_cmp.wast**, **f32_bitwise.wast** - F32 operation tests
- **f64.wast**, **f64_cmp.wast**, **f64_bitwise.wast** - F64 operation tests
- **float_exprs.wast**, **float_literals.wast**, **float_memory.wast**, **float_misc.wast** - Floating point tests

#### Proposal Subdirectories
- **bulk-memory/** - Bulk memory operations tests
- **exceptions/** - Exception handling tests
- **gc/** - Garbage collection tests
- **memory64/** - 64-bit memory tests
- **multi-memory/** - Multiple memory tests
- **relaxed-simd/** - Relaxed SIMD tests
- **simd/** - SIMD operations tests

### Test Infrastructure
- **run.py** - Test runner script
- **README.md** - Test suite documentation

## Proposals (`proposals/`)

### Active Proposals
- **simd/** - SIMD operations (vector instructions)
  - Overview.md, SIMD.md, BinarySIMD.md, TextSIMD.md
- **gc/** - Garbage collection (reference types, structs, arrays)
  - MVP.md, Post-MVP.md, Overview.md, Charter.md
- **exception-handling/** - Exception handling
  - Exceptions.md, Legacy exceptions
- **function-references/** - Function references
  - Overview.md
- **reference-types/** - Reference types
  - Overview.md
- **multi-memory/** - Multiple memories
  - Overview.md
- **memory64/** - 64-bit memory addressing
  - Overview.md
- **tail-call/** - Tail call optimization
  - Overview.md

### Other Proposals
- **bulk-memory-operations/** - Bulk memory operations
- **multi-value/** - Multiple return values
- **nontrapping-float-to-int-conversion/** - Non-trapping conversions
- **sign-extension-ops/** - Sign extension operations
- **extended-const/** - Extended constant expressions
- **annotations/** - Custom annotations
- **branch-hinting/** - Branch hints
- **relaxed-simd/** - Relaxed SIMD semantics
- **js-string-builtins/** - JavaScript string builtins

## Reference Implementation (`interpreter/`)

### Core Components
- **runtime/** - Runtime system implementation
- **exec/** - Instruction execution engine
- **syntax/** - Parser and AST
- **binary/** - Binary format decoder
- **text/** - Text format parser
- **valid/** - Module validator
- **host/** - Host function interface
- **custom/** - Custom section handling

### Build Files
- **dune**, **dune-project** - OCaml build configuration
- **Makefile** - Build instructions
- **wasm.opam** - OPAM package definition
- **LICENSE** - Interpreter license
- **README.md** - Implementation documentation

## Specification Tooling (`spectec/`)

### Components
- **src/** - Source code for spec generation tools
- **doc/** - Tooling documentation
  - Overview.md, Language.md, Interpreter.md, Prose.md, Latex.md
  - IL.md, EL.md, Splicing.md, Usage.md, Assumptions.md
- **spec/** - Specification source files
- **test-frontend/** - Frontend tests
- **test-middlend/** - Middle-end tests
- **test-interpreter/** - Interpreter tests
- **test-latex/** - LaTeX generation tests
- **test-prose/** - Prose generation tests
- **test-splice/** - Splicing tests

### Build Files
- **Makefile**, **dune**, **dune-project** - Build configuration
- **README.md** - Tooling documentation

## Research Papers (`papers/`)

- **pldi2017.pdf** - "Bringing the Web up to Speed with WebAssembly" (2017)
- **oopsla2019.pdf** - "Verifying WebAssembly Safety" (2019)
- **pldi2024.pdf** - Latest WebAssembly research (2024)
- **README.md** - Papers documentation
- **LICENSE** - Papers license

## Key Concepts Reference

### Module vs Module Instance
- **Module** (static): The compilation unit containing definitions (types, functions, globals, memories, tables, data segments, imports, exports)
- **Module Instance** (runtime): The actual runtime representation created during instantiation, containing addresses to runtime instances

### Data Segments
- **Active**: Copies contents into memory during instantiation (offset from constant expression)
- **Passive**: Contents copied later via `memory.init` instruction

### Constant Expressions
Used for initialization values in globals, tables, data/element segments. Limited to:
- `CONST` instructions
- `REF.NULL`, `REF.FUNC`
- `GLOBAL.GET` (imports or previous globals)
- Struct/array constructors
- **Cannot call functions or affect the store**

### Key Specification Files for Implementation
1. **syntax/modules.rst** - Understand module structure
2. **exec/runtime.rst** - Understand runtime objects (store, instances)
3. **exec/modules.rst** - Understand instantiation process
4. **valid/instructions.rst** - Understand validation rules (constant expressions)
5. **exec/instructions.rst** - Understand instruction execution
6. **binary/modules.rst** - Understand binary encoding

### Testing
- Use **test/core/** files as reference for expected behavior
- Run tests with **test/core/run.py**
- Proposal-specific tests in **test/core/[proposal]/** subdirectories
