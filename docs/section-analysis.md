# WASM Section Analysis

This document analyzes the WebAssembly binary format sections to inform implementation strategy.

## Research Questions

1. **Section Classification**: Which sections contain pure declarations vs. actual data?
2. **Block Instructions**: How should nested control flow be represented and parsed?

## Question 1: Section Classification

### Pure Declarations (Metadata Only)

These sections contain only type information, indices, or references to other sections:

| Section | ID | Contents | Purpose |
|---------|----|----|---------|
| **Type** | 1 | Function type signatures | Type metadata for validation |
| **Function** | 3 | Type indices (u32) | Maps function → type (bodies in Code section) |
| **Import** | 2 | Module/name/descriptor tuples | External declarations (func/table/mem/global) |
| **Export** | 7 | Name + export descriptor (index) | Export metadata pointing to runtime items |
| **Start** | 8 | Single function index | Module initialization entry point |
| **Data Count** | 12 | Optional u32 count | Validation hint for single-pass parsing |

**Key insight**: The Function section is separated from the Code section to enable **parallel and streaming compilation** (spec note at line 9). Functions can be validated before their bodies are parsed.

### Sections with Real Data

These sections contain executable code, initialization expressions, or data payloads:

| Section | ID | Contents | Data Type |
|---------|----|----|-----------|
| **Code** | 10 | Function bodies (locals + instructions) | Executable bytecode |
| **Global** | 6 | GlobalType + init expression | Type + initialization code |
| **Element** | 9 | Element segments with init exprs | Table initialization data |
| **Data** | 11 | Data segments with byte vectors | Memory initialization data |
| **Table** | 4 | Table types (reftype + limits) | Runtime size constraints |
| **Memory** | 5 | Memory types (limits) | Runtime size constraints |

**Initialization expressions** (`Expr`) appear in:
- Global section: `{ type, init_expr }` 
- Element section: `{ type, init: Vec<Expr>, mode }`
- Data section: `{ init: Vec<u8>, mode }`

Active segments include an offset expression evaluated at instantiation time.

### Implementation Priority

**Phase 1 - Simple Parsing** (declarations):
1. ✅ Type Section (pure function signatures)
2. ✅ Function Section (type indices)
3. Import Section (name + descriptor parsing)
4. Export Section (name + descriptor parsing)
5. Start Section (single index)

**Phase 2 - Data Sections** (requires expression parser):
6. ✅ Code Section (locals + instruction sequences)
7. Global Section (needs constant expression evaluation)
8. Element Section (needs expression parsing)
9. Data Section (byte vectors + expressions)

**Phase 3 - Advanced**:
10. Table/Memory sections (already have type parsers)
11. Data Count Section (simple u32)

## Question 2: Block Instructions - Nested Structure vs Streaming

### Binary Format: Flat Byte Stream

From spec `binary/instructions.rst` lines 50-58, block instructions are **not nested in binary**:

```
Block:  0x02 <blocktype> (instructions)* 0x0B
Loop:   0x03 <blocktype> (instructions)* 0x0B
If:     0x04 <blocktype> (instructions)* 0x0B
        0x04 <blocktype> (instructions)* 0x05 (instructions)* 0x0B
```

**Key characteristics:**
- Flat sequence of opcodes
- Explicit end markers: `0x0B` (END), `0x05` (ELSE)
- No length prefixes or nesting indicators
- Parser must track block depth while scanning

### Abstract Syntax: Nested Structure

In the abstract syntax (text format), blocks ARE nested:

```wasm
(block (result i32)
  (i32.const 1)
  (i32.const 2)
  (i32.add)
)
```

Corresponds to our current AST:
```rust
Instr::Block(BlockType, Vec<Instr>)
Instr::Loop(BlockType, Vec<Instr>)
Instr::If(BlockType, Vec<Instr>, Vec<Instr>)
```

### The Correspondence Problem

**Binary representation:**
```
[0x02, 0x40, 0x41, 0x01, 0x41, 0x02, 0x6A, 0x0B]
 ^^^^  ^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^  ^^^^
 BLOCK TYPE  nested instructions          END
```

**Two parsing strategies:**

#### Strategy A: Parse to AST (current approach)
```rust
// Parse nested structure during binary parsing
parse_block() {
    let bt = parse_blocktype();
    let instrs = parse_instructions_until_end();  // Recursive
    Instr::Block(bt, instrs)
}
```

**Pros:**
- Clean AST matches text format
- Easy to pretty-print and debug
- Works well with recursive interpreters

**Cons:**
- Requires full parse before execution
- Memory overhead from Vec allocations
- Need to flatten for streaming execution

#### Strategy B: Streaming Interpretation (address-based)
```rust
// Keep binary flat, track block addresses at runtime
struct LabelFrame {
    end_addr: usize,      // Jump target for 'br' and 'end'
    continuation: usize,  // For 'loop', jump back here
    arity: usize,         // How many values to keep on break
}

// During execution:
match opcode {
    0x02 => {  // block
        let bt = read_blocktype();
        labels.push(LabelFrame { 
            end_addr: find_matching_end(pc),  // Scan forward
            arity: bt.results().len()
        });
    }
    0x0B => {  // end
        labels.pop();
    }
    0x0C => {  // br n
        let frame = labels[labels.len() - 1 - n];
        pc = frame.end_addr;
    }
}
```

**Pros:**
- True streaming - can execute as we parse
- Minimal memory overhead
- Matches actual binary structure
- Efficient for long-running code

**Cons:**
- Need to scan forward to find `0x0B` for each block entry
- More complex runtime state
- Harder to debug (no AST to inspect)

### Hybrid Strategy (Recommended)

**For wasmly.rs**: Keep the current AST-based approach but be aware of the correspondence:

1. **Parse to AST** (current): 
   - Use `Instr::Block(bt, Vec<Instr>)` representation
   - Matches the abstract syntax and text format
   - Easy to test, debug, and validate

2. **Execution**: 
   - Tree-walking interpreter works directly on AST
   - For streaming optimization later, can maintain a "flattened view" with position tracking

3. **Future optimization path**:
   - Add `instruction_offset` field to track binary position
   - Build address-based label stack during execution
   - Enable JIT compilation by keeping binary correspondence

### Block Parsing Implementation

Current implementation in `src/binary/sections.rs`:

```rust
pub fn parse_instructions(input: Input) -> ParseResult<'_, Vec<Instr>> {
    let mut remaining = input;
    let mut instrs = Vec::new();
    
    loop {
        let (rest, opcode) = parse_byte(remaining)?;
        remaining = rest;
        
        match opcode.value {
            0x0B => break,  // end - terminates current block
            0x02 => {  // block
                let (rest, bt) = parse_blocktype(remaining)?;
                let (rest, nested) = parse_instructions(rest)?;  // Recursive!
                remaining = rest;
                instrs.push(Instr::Block(bt, nested));
            }
            // ... other instructions
        }
    }
    
    Ok((remaining, instrs))
}
```

**This is correct** for AST-based interpretation. The recursion naturally handles nesting.

### Expression vs Instruction Sequence

From spec line 393-394 (Code Section):

> The function *body* as an :ref:`expression <binary-expr>`.

An `Expr` is an instruction sequence terminated by `0x0B`:

```rust
pub struct Expr {
    pub instrs: Vec<Instr>,
}

pub fn parse_expr(input: Input) -> ParseResult<'_, Expr> {
    let (remaining, instrs) = parse_instructions(input)?;
    Ok((remaining, Expr { instrs }))
}
```

The `0x0B` at the end of a function body is consumed by `parse_instructions`, so `Expr` contains just the instructions without the explicit END marker.

## Recommendations

### For Section Parsing

1. **Next implementation targets** (in order):
   - Import Section (straightforward: name + descriptor)
   - Export Section (straightforward: name + descriptor)
   - Memory/Table Sections (types already implemented)
   - Start Section (single index)
   - Data Count Section (single u32)

2. **Defer until expression parser is complete**:
   - Global Section (needs const expression evaluator)
   - Element Section (complex with 8 encoding variants)
   - Data Section (needs offset expressions)

### For Block Instructions

1. **Keep current AST approach** - it's correct and maintainable

2. **Be aware of the correspondence**:
   - Binary: flat with explicit `0x0B` markers
   - AST: nested `Vec<Instr>` structures
   - Parsing bridge: recursive `parse_instructions()` call

3. **Future optimization** (if needed):
   - Add `binary_offset` metadata to `Instr` enum
   - Implement address-based label stack for execution
   - Enable streaming validation

4. **Testing strategy**:
   - Test block nesting with WASM spec tests
   - Verify `0x0B` consumption at each nesting level
   - Test `br` (branch) instructions with correct label depths

## References

- WASM Spec: `wasm-spec/document/core/binary/modules.rst`
- WASM Spec: `wasm-spec/document/core/binary/instructions.rst` (lines 24-73)
- Note at line 9: "This separation enables *parallel* and *streaming* compilation"
- Code Section: lines 370-425
- Block types: lines 43-46
- Control instructions: lines 47-66
