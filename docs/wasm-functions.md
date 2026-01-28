# WebAssembly Functions

This document summarizes the WebAssembly specification for functions, including their structure, signatures, code blocks, locals, validation, and relationship to function tables.

## Function Structure

A function in WebAssembly consists of three parts:

```
func ::= { type: typeidx, locals: vec(valtype), body: expr }
```

- **`type`** (`FTYPE`): Index into the Type Section referencing a function type
- **`locals`** (`FLOCALS`): Vector of local variable declarations (each is a value type)
- **`body`** (`FBODY`): Expression - an instruction sequence terminated by END

## Function Signatures

Function types define the signature in the Type Section:

```
functype ::= [t1*] -> [t2*]
```

- **`[t1*]`**: Parameter types (inputs to the function)
- **`[t2*]`**: Return types (outputs from the function)

### Example

A function type `[i32, f32] -> [i64]` describes:
- Takes two parameters: an `i32` and an `f32`
- Returns one `i64` value

All function types used in a module must be defined in the Type Section and are referenced by type indices.

## Binary Format

Functions are split across two separate sections in the binary format to enable parallel/streaming compilation.

### Function Section (id=3)

Contains only type indices for each function:

```
funcsec ::= typeidx* : vec(typeidx)
```

- One entry per function in the module
- Each entry is a `typeidx` referencing a function type from the Type Section
- The `FTYPE` field of each function

### Code Section (id=10)

Contains the locals declarations and body expressions for each function:

```
codesec ::= code* : vec(code)
code   ::= size : u32, func
func    ::= (t*)* : vec(locals), e : expr
locals  ::= n : u32, t : valtype
```

Each code entry:
1. **`size`**: Byte size of the function code (allows skipping during streaming)
2. **`locals`**: Compressed vector of `(count, type)` pairs
   - Each pair declares `count` locals of type `valtype`
   - Example: `[(2, i32), (1, f64)]` declares 2 i32 locals + 1 f64 local = 3 total
3. **`body`**: Expression (instruction sequence) terminated by END

### Length Matching

The Function Section and Code Section MUST have the same length. They are paired by position:
- Function 0 uses `typeidx` from Function Section[0] + locals/body from Code Section[0]
- Function 1 uses `typeidx` from Function Section[1] + locals/body from Code Section[1]
- And so on...

## Locals

### Local Declaration and Indexing

The locals of a function include both parameters and explicitly declared local variables:

1. **Parameters from signature**: First locals (indices 0, 1, 2, ...)
   - Number of parameters = length of `[t1*]` from the function type
   - These are immutable during validation but mutable at runtime via `local.set`

2. **Explicit locals**: Follow the parameters
   - Declared in the code section's `locals` vector
   - Each `(n, t)` pair adds `n` locals of type `t`

### Example

If a function has:
- Signature: `[i32, i32] -> [i64]` (2 parameters)
- Locals declaration: `[(3, i32), (1, f64)]` (4 explicit locals)

The local index space is:
- Index 0: Parameter 1 (i32)
- Index 1: Parameter 2 (i32)
- Index 2-4: 3 i32 locals
- Index 5: 1 f64 local

Total: 6 locals accessible via indices 0-5.

### Local Mutability

All locals (including parameters) are **mutable**:
- `local.get x`: Read value from local at index `x`
- `local.set x`: Write value to local at index `x`
- `local.tee x`: Write value to local at index `x` AND keep value on stack

## Local Validation

When validating a function's body, a specific context is constructed (valid/modules.rst:20-39):

1. Create context `C'` with:
   - `CLOCALS` = parameters + locals (concatenated)
   - `CLABELS` = `[return_types]` (singular label for the function body)
   - `CRETURN` = `[return_types]` (expected return type)

2. Validate the body expression under this context

3. The body must produce a stack matching the return types `[t2*]`

### Variable Instruction Validation

All variable instructions validate against the local types in the context:

**`local.get x`** (valid/instructions.rst:659-676):
- Requires: `C.CLOCALS[x]` exists (local is defined)
- Type: `[] -> [t]` where `t = C.CLOCALS[x]`
- Pops nothing, pushes a value of the local's type

**`local.set x`** (valid/instructions.rst:678-695):
- Requires: `C.CLOCALS[x]` exists (local is defined)
- Type: `[t] -> []` where `t = C.CLOCALS[x]`
- Pops a value of type `t`, pushes nothing

**`local.tee x`** (valid/instructions.rst:697-714):
- Requires: `C.CLOCALS[x]` exists (local is defined)
- Type: `[t] -> [t]` where `t = C.CLOCALS[x]`
- Pops a value, writes to local, pushes the same value back

### Runtime Execution (exec/instructions.rst:1072-1130)

At runtime, the function's activation frame holds all locals in `F.ALOCALS`:

- `local.get x`: Pushes `F.ALOCALS[x]` to stack
- `local.set x`: Pops value `val`, sets `F.ALOCALS[x] = val`
- `local.tee x`: Pops value `val`, sets `F.ALOCALS[x] = val`, pushes `val`

## Function Tables and Indirect Calls

### Table Structure

Tables store function references (`funcref`) to enable dynamic function calls:

```
table        ::= { type: tabletype }
tableinst    ::= { type: tabletype, elem: vec(funcref) }
tabletype    ::= { min: u32, max: u32?, reftype }
```

A table instance:
- Has a fixed type specifying element type (`funcref`) and limits
- Contains a vector of function references
- Elements can be mutated via `table.set` and read via `table.get`

### CALLINDIRECT Instruction

```
call_indirect x y
```

- **`x`**: Table index (which table to use)
- **`y`**: Type index (expected function type)

**Execution semantics** (exec/instructions.rst:2839-2906):

1. Pop an `i32` index `i` from the stack
2. Bounds check: if `i >= table.length`, trap
3. Get reference `r = table.elem[i]`
4. Null check: if `r` is `ref.null`, trap
5. Extract function address `a` from `ref.funcaddr r`
6. Get function instance `f = S.SFUNCS[a]`
7. **Dynamic type check**: if `f.FITYPE != expected_type`, trap
8. Invoke function at address `a`

### Purpose of Function Tables

Function tables enable:
- **Function pointers**: Store function addresses in tables, call them indirectly
- **Callbacks**: Pass functions as values between different parts of code
- **Dynamic dispatch**: Select which function to call at runtime based on data
- **Polymorphism**: Write generic code that operates on different function types

The key innovation is the **runtime type check**: even though the function is selected dynamically, the signature must match the expected type, providing type safety.

### Table Initialization

Tables are initialized via **element segments** in the binary format:

- **Active elements**: Copy function references into table during module instantiation
- **Passive elements**: Can be copied to table later via `table.init` instruction
- **Declarative elements**: Forward-declare functions that can be referenced but aren't available at runtime

Element segments contain lists of constant expressions (typically `ref.func funcidx`) that evaluate to function references.

## References

- Syntax: `wasm-spec/document/core/syntax/modules.rst` (functions, tables, indices)
- Binary format: `wasm-spec/document/core/binary/modules.rst` (function/code sections, encoding)
- Validation: `wasm-spec/document/core/valid/modules.rst` (function/locals validation)
- Execution: `wasm-spec/document/core/exec/instructions.rst` (local.get/set, call_indirect)
- Runtime: `wasm-spec/document/core/exec/runtime.rst` (frames, table instances, function instances)
