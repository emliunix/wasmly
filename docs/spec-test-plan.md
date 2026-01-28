# Spec Test Plan (wasmly.rs)

Date: 2026-01-21

This document is a **project test plan** focused on compliance with the official WebAssembly specification test suite shipped in `./wasm-spec/`.

Scope priority for wasmly.rs (current focus):

1. **Binary parsing** (.wasm decoding)
2. **Module validation** (static semantics, index-space checks, type checking)
3. **Instantiation & execution** (runtime semantics)

> Notes about the local checkout
>
> - In this repo, `wasm-spec/test/core/` contains **147 `.wast` scripts** (plus `README.md`, `run.py`, and `simd/`).
> - There are no precompiled `.wasm` fixtures in that directory.
> - `.wast` scripts contain a mixture of:
>   - text modules `(module (func ...) ...)` (WAT embedded)
>   - inline binary modules `(module binary "\\00asm..." ...)`
>   - script directives and assertions (`assert_malformed`, `assert_invalid`, `assert_return`, `invoke`, ...)

---

## 1. Goals

### 1.1 Compliance goals

- **Decode** all valid modules in target test subsets.
- **Reject malformed binaries** with correct class of failure (malformed vs invalid).
- **Validate** modules according to the spec’s validation rules.
- Later: **execute** a growing subset of semantics tests.

### 1.2 Non-goals (initially)

- Full host/import environment emulation for every spec test.
- Full floating-point edge-case compliance before integer/control-flow baseline is stable.
- SIMD proposal tests (`test/core/simd/*`) until MVP integer/core parsing is correct.

---

## 2. Harness strategy (how we consume wasm-spec tests)

We use **two ingestion modes**:

### Mode A — Inline binary extraction (best for binary parser)

Used for `.wast` scripts with `(module binary "..." ...)`, e.g.

- `binary.wast`
- `binary-leb128.wast`
- `custom.wast`

Advantages:
- No external toolchain required.
- Many tests explicitly check malformed binary corner cases.

### Mode B — Compile `.wast`/text modules to `.wasm` first

Used for `.wast` scripts with text modules and runtime assertions (e.g. `nop.wast`).

Recommended toolchain approach:

- **WABT** `wast2json` to compile/extract modules:
  - Produces `*.wasm` files + a JSON manifest of actions/assertions.
  - We can initially ignore assertions and use the extracted `.wasm` for decoding + validation tests.

---

## 3. Test suite categorization

This section categorizes the `.wast` files into buckets relevant to wasmly.

### 3.1 Binary format & encoding (decoder-focused)

Primary: ensure the binary parser correctly handles:
- magic/version
- section id legality
- section length boundaries
- LEB128 decoding (including non-minimal encodings)
- custom sections / name sections / utf-8 corner cases

Candidate files:
- `binary.wast` (high)
- `binary-leb128.wast` (high)
- `custom.wast` (high)
- `endianness.wast` (medium)
- `names.wast` (medium)
- `utf8-custom-section-id.wast` (medium)
- `utf8-import-field.wast` (medium)
- `utf8-import-module.wast` (medium)
- `utf8-invalid-encoding.wast` (medium)

### 3.2 Section coverage (decoder + validator)

Goal: parse each section payload into `crate::module::*` structures, and validate index spaces.

Candidate files:
- Type/import/export/start/global/table/memory/elem/data focused:
  - `type.wast`, `imports.wast`, `exports.wast`, `start.wast`, `global.wast`,
  - `table.wast`, `memory.wast`, `elem.wast`, `data.wast`,
  - plus: `linking.wast`, `names.wast`.

### 3.3 Instruction decoding (decoder)

Goal: expand opcode coverage for `Instr` decoding.

Candidate files (instruction-operator families):
- Control flow: `block.wast`, `loop.wast`, `if.wast`, `br.wast`, `br_if.wast`, `br_table.wast`, `return.wast`, `unreachable.wast`
- Locals: `local_get.wast`, `local_set.wast`, `local_tee.wast`
- Calls: `call.wast`, `call_indirect.wast`, `func.wast`, `func_ptrs.wast`
- Numeric: `i32.wast`, `i64.wast`, `f32.wast`, `f64.wast`, `conversions.wast`, `const.wast`
- Memory ops & alignment: `load.wast`, `store.wast`, `align.wast`, `address.wast`
- Tables/references: `table_*.wast`, `ref_null.wast`, `ref_is_null.wast`, `ref_func.wast`
- Bulk memory: `bulk.wast`, `memory_copy.wast`, `memory_fill.wast`, `memory_init.wast`, `table_copy.wast`, `table_init.wast`, `table_fill.wast`

### 3.4 Validation/error-classification (validator)

Goal: distinguish:
- **malformed** (binary decode errors)
- **invalid** (validation errors)

Candidate files:
- `unreached-invalid.wast`
- `unreached-valid.wast`
- `traps.wast` (later: runtime traps)
- `token.wast`, `comments.wast`, `obsolete-keywords.wast` (mainly script/text parsing)

### 3.5 Runtime semantics (executor)

Goal: execute exported functions and satisfy `assert_return`, `assert_trap`.

Candidate files (start small):
- `nop.wast`, `const.wast` (simple)
- `fac.wast` (small “real program”)
- `left-to-right.wast`, `stack.wast`, `switch.wast` (harder sequencing/control)
- Floating-point suites are large and sensitive: `f32_*`, `f64_*` (defer until later)

### 3.6 Proposals / extensions

SIMD proposal tests live in `wasm-spec/test/core/simd/*.wast`.

Plan:
- Track separately under an “extensions” milestone.
- Don’t block MVP compliance on SIMD.

---

## 4. Milestones

### Milestone M0 — Binary header + section iteration

Success criteria:
- Parse magic/version.
- Iterate through sections, accept custom sections anywhere.
- Enforce section id legality and length bounds.

Target inputs:
- `binary.wast` (header + section id/length malformed cases)
- `custom.wast`

### Milestone M1 — Type section + function/code scaffold

Success criteria:
- Parse type section.
- Parse function section and code section structure (even if instruction set is partial).

Target inputs:
- `type.wast`
- `binary.wast` function/code snippets

### Milestone M2 — Instruction decode expansion

Success criteria:
- Decode the opcode set required by: `nop.wast`, `local_*.wast`, `block/loop/if/br*.wast`, basic i32 arithmetic.

### Milestone M3 — Validation MVP

Success criteria:
- Basic validation: section ordering rules, index bounds, function/code length match, type checking (as implemented).
- Categorize failures into malformed vs invalid in test harness.

### Milestone M4 — Minimal execution

Success criteria:
- Instantiate module with no imports.
- Execute a subset of exports (i32-only), satisfying `assert_return`.

---

## 5. Status tracking

We track compliance at two levels:

1) **Category status** (coarse): how complete is parsing/validation/execution for a bucket?
2) **File status** (fine): per `.wast` file whether we pass, partially pass, or are blocked.

### 5.1 Status legend

- ✅ PASS
- 🟨 PARTIAL (some directives pass, others skipped/fail)
- ❌ FAIL
- ⏭️ SKIP (not targeted yet)
- 🚧 WIP (in progress)

### 5.2 Category status (template)

| Category | Parse (.wasm) | Validate | Execute | Notes |
|---|---:|---:|---:|---|
| Binary format & encoding | 🚧 | ⏭️ | ⏭️ | Start with inline `(module binary ...)` cases |
| Section coverage | 🚧 | ⏭️ | ⏭️ | Type/function/code first |
| Instruction decoding | 🚧 | ⏭️ | ⏭️ | Expand opcode coverage iteratively |
| Validation classification | ⏭️ | ⏭️ | ⏭️ | Need malformed vs invalid mapping |
| Runtime semantics | ⏭️ | ⏭️ | ⏭️ | i32-only first |
| SIMD | ⏭️ | ⏭️ | ⏭️ | Separate milestone |

### 5.3 File status (starter list)

> Keep this table updated as we implement. Add more files as we start covering them.

| File | Category | Harness mode | Priority | Parse | Validate | Exec | Notes |
|---|---|---|---:|---:|---:|---:|---|
| `binary.wast` | Binary format & encoding | A | P0 | 🚧 | ⏭️ | ⏭️ | Inline binary + malformed cases |
| `binary-leb128.wast` | Binary format & encoding | A | P0 | 🚧 | ⏭️ | ⏭️ | Non-minimal encodings allowed |
| `custom.wast` | Binary format & encoding | A | P0 | 🚧 | ⏭️ | ⏭️ | Custom sections anywhere |
| `type.wast` | Section coverage | B | P0 | ⏭️ | ⏭️ | ⏭️ | Compile/extract modules |
| `imports.wast` | Section coverage | B | P1 | ⏭️ | ⏭️ | ⏭️ | Host env required for exec |
| `exports.wast` | Section coverage | B | P1 | ⏭️ | ⏭️ | ⏭️ | |
| `nop.wast` | Runtime semantics | B | P1 | ⏭️ | ⏭️ | ⏭️ | Good for early exec once decoding works |
| `i32.wast` | Instruction decoding | B | P1 | ⏭️ | ⏭️ | ⏭️ | Large but fundamental |
| `block.wast` | Instruction decoding | B | P1 | ⏭️ | ⏭️ | ⏭️ | Control-flow |
| `simd/*` | SIMD | B | P3 | ⏭️ | ⏭️ | ⏭️ | Defer |

---

## 6. Practical test commands

- Unit tests: `cargo test`
- Focused runs:
  - `cargo test binary` (once we name tests accordingly)
  - `cargo test spec::binary::` (suggested module naming)

---

## 7. Next actions (implementation-facing)

- Implement a minimal harness for Mode A (inline `(module binary ...)`) and wire it into `cargo test`.
- Expand the parser to decode:
  - section headers robustly
  - type section
  - function + code section structure
- Add WABT-based extraction (Mode B) once we want broader coverage beyond `binary*.wast`.
