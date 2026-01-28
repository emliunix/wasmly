# WAST Test Framework Plan (wasmly.rs)

Date: 2026-01-21

This document proposes a plan to implement a **`.wast` test framework** for wasmly.rs, integrating the WebAssembly spec test suite (`./wasm-spec/test/core/*.wast`) with `cargo test`.

Focus:
- Parse and run **WAST scripts** (S-expression test scripts used by the spec interpreter).
- Extract/compile modules to `.wasm` bytes.
- Feed bytes into wasmly’s **binary parser** (primary near-term goal).
- Gradually add **validation** and **execution** of `assert_*` directives.

---

## 1. Background: what `.wast` is

`.wast` in the spec suite is not just “WAT”. It’s a script language that contains:

- Module definitions:
  - text form: `(module (func ...) ...)` (WAT embedded)
  - binary literal: `(module binary "\\00asm..." ...)`
- Commands/directives:
  - registration, invoking exports, getting globals
- Assertions:
  - `assert_malformed`, `assert_invalid`, `assert_return`, `assert_trap`, etc.

Therefore, a `.wast` runner is closer to a **test harness** than a compiler.

---

## 2. Approaches (choose one as primary)

### Option A — Use WABT `wast2json` (fastest path to large coverage)

Workflow:
1. Run `wast2json file.wast -o out/file.json`
2. WABT emits:
   - one or more `out/file.N.wasm`
   - a JSON manifest describing directives/assertions
3. Our harness:
   - parses each emitted `.wasm` using wasmly’s binary parser (MVP)
   - later interprets JSON to run assertions

Pros:
- Offloads WAST parsing and WAT→WASM compilation.
- Produces stable module artifacts.

Cons:
- Requires external dependency (wabt) in CI/dev.
- JSON schema ties us to WABT behavior.

### Option B — Use Rust crates (`wast`/`wat`) and stay in-process (best integration)

Recommended if we want a “pure Rust” harness integrated with `cargo test`.

Likely building blocks:
- `wast` crate (from Bytecode Alliance / wasm-tools ecosystem): parse `.wast` scripts into an AST of directives with spans.
- `wat` crate: compile WAT text modules to `.wasm` bytes.

Pros:
- Great integration with Rust error reporting (spans, file/line/col).
- No external tool required.

Cons:
- Requires implementing the execution/validation side ourselves.

### Option C — Shell out to spec interpreter `run.py` / `interpreter/wasm`

Pros:
- Maximum spec faithfulness.

Cons:
- Harder to integrate with wasmly’s internal error types.
- Doesn’t directly test *our* parser unless we build hooks.

**Plan assumes Option B as the long-term direction**, while keeping Option A as a pragmatic bridge for high coverage.

---

## 3. Proposed architecture (Option B)

### 3.1 Crate/module layout

- `src/wast/` (or `src/spec/`)
  - `loader.rs`: read files, parse scripts
  - `runner.rs`: execute directives + assertions
  - `report.rs`: convert failures into nice `cargo test` output

Or: create a separate dev-only crate `crates/spec-harness`.

### 3.2 Core data types

- `TestCase`:
  - `file: PathBuf`
  - `directive_index: usize`
  - `name: String` (stable subtest name)
  - `directive: ...` (parsed AST node)

- `HarnessConfig`:
  - which categories/files enabled
  - allow-skips
  - strictness toggles (e.g. accept non-minimal LEB128)

- `HarnessError`:
  - includes:
    - `.wast` span (line/col)
    - optional byte offset into generated `.wasm`
    - wasmly `BinaryError` / parse error

---

## 4. Execution model (what we can support over time)

### Phase 0 — Parse scripts and enumerate directives

- Parse `.wast` file
- Enumerate directives
- Create one Rust test per file (or per directive)

Deliverable: `cargo test` lists tests like:
- `spec::core::binary::file_binary_wast`
- `spec::core::nop::directive_012_assert_return`

### Phase 1 — Binary parser compliance only

Target directives:
- `(module binary ...)`
- `(module ...)` (text module) compiled to `.wasm` bytes

Checks:
- For “expected success” modules: wasmly parser returns `Ok`
- For `assert_malformed`: wasmly parser returns `Err` (and optionally error class matches)

> At this phase, we do NOT need a runtime executor.

### Phase 2 — Validation

Add a wasmly validation layer (even partial) and map WAST assertions:
- `assert_invalid` → decode OK + validate FAIL
- `assert_malformed` → decode FAIL

### Phase 3 — Minimal execution

Implement enough instantiation/execution to cover:
- `assert_return (invoke ...) (i32.const ...)`
- `assert_trap (invoke ...) "..."`

Start with:
- modules with no imports
- integer-only ops

---

## 5. Integration with `cargo test`

### 5.1 Test discovery / parameterization

Rust’s built-in test harness doesn’t support dynamic test generation in stable without help.

Recommended options:

1) **`datatest-stable`**
- Creates one test per file matching a glob.
- Great for per-`.wast` tests.

2) **`libtest-mimic`**
- Full control over generating one test per directive.
- Best granularity (each assertion becomes one subtest).

3) “One test per file” using plain `#[test]`
- Easiest; but failures don’t isolate which directive failed.

Suggested:
- Use `datatest-stable` for MVP (one test per `.wast`).
- Upgrade to `libtest-mimic` when we want per-directive reporting.

### 5.2 Error reporting quality

Goals:
- Show `.wast` file + line/column of the failing directive.
- Show wasmly parse error with byte offset (`SourceLocation.offset`) when available.
- Provide context (which directive, which embedded module).

Implementation ideas:
- Use the span information from the `wast` parser.
- Use `miette` (optional) to display “diagnostic” style errors.
- Include a hexdump window around the failing byte offset for binary parse errors.

### 5.3 Skips and expected failures

We need a way to mark tests as:
- `ignored` (not targeted yet)
- `known failing`

Approach:
- Maintain a `spec-status.toml` or `spec-status.json` mapping:
  - file → status
  - file+directive index → status

This can drive `#[ignore]` or runtime skip logic.

---

## 6. How to compile modules from `.wast`

We have 3 source forms:

1) `(module binary "...")`
- We must decode the string escape sequences into raw bytes.
- This is already described in `docs/wasm-spec-tests.md`.

2) `(module (func ...) ...)` (text module)
- Use `wat` crate to compile WAT text → `.wasm` bytes.

**Practical note:** the Rust `wast` crate represents module payloads as `QuoteWat`, and provides
`QuoteWat::to_test()` which returns either:

- `QuoteWatTest::Binary(Vec<u8>)` (ready-to-parse `.wasm` bytes)
- `QuoteWatTest::Text(String)`

This is a convenient bridge because it centralizes the logic for handling `module`, `module binary`, and `module quote` forms.

3) Multi-module scripts
- The runner must maintain a registry of instantiated modules (`$M1`, `$M2`, etc.)
- Needed for `register`, `invoke`, cross-module tests.

---

## 7. Minimal API surface for the harness

Proposed entrypoints:

- `run_wast_file(path: &Path, cfg: &HarnessConfig) -> HarnessReport`
- `run_wast_directive(...) -> Result<(), HarnessError>`

Where `HarnessReport` tracks:
- number of directives
- passed/failed/skipped counts
- detailed failures

---

## 8. Concrete next steps

1) Pick MVP integration style:
   - Start with **one test per `.wast` file** under `tests/spec_core.rs`.

2) Implement WAST parsing (Option B):
   - Add dependency on `wast` (and `wat`) crates.
   - Parse a `.wast` file and iterate directives.

3) Implement module extraction:
   - `module binary` → bytes
   - `module` text → compile to bytes

4) Hook bytes into wasmly binary parser:
   - `wasmly::binary` parsers return `Result` with locations.

5) Add a status/skip mechanism:
   - start with hardcoded allowlist: `binary.wast`, `binary-leb128.wast`, `custom.wast`.

---

## 9. Parity with `wasm-spec/test/core/run.py`

The upstream spec repository provides a small Python runner at:

- `wasm-spec/test/core/run.py`

It is **not** the interpreter itself. It is a *test orchestrator* built on Python `unittest` that shells out to the spec interpreter executable `wasm`.

### 9.1 What `run.py` actually does

For each `*.wast` file (including `simd/*.wast`), `run.py` performs roughly:

1. **Generate JS**
   - `wasm -d <input.wast> -o <output.js>`

2. **Optionally stop** if `--generate-js-only` is set

3. **Run original script**
   - `wasm <input.wast>`
   - Expected exit code:
     - `0` normally
     - `1` if the filename contains `.fail.`

4. If run succeeded, perform **round-trip conversions**:
   - Convert script to “binary-script” and run:
     - `wasm -d <input.wast> -o <output.bin.wast>`
     - `wasm <output.bin.wast>`
   - Convert back to text and run:
     - `wasm -d <output.bin.wast> -o <output.bin.wast.wast>`
     - `wasm <output.bin.wast.wast>`
   - Convert again and **compare** the conversion outputs for stability:
     - Compare the two generated `.bin.wast` files
     - Compare the two generated `.wast` files

5. Optional JS execution:
   - If `--js <cmd>` is provided, run `<cmd> <output.js>`

Implementation notes:
- It writes command output to `*_output/*.log` files.
- It uses `--out` to choose the output directory.
- It uses `--failfast` to stop on first failure.

### 9.2 How the spec interpreter (`wasm`) handles `.wast` internally

The reference interpreter has a clear split between **parsing** the script language and **running** it.

Key OCaml modules (in `wasm-spec/interpreter/`):

- `text/lexer.mll`, `text/parser.mly`:
  - Defines the `.wat` and `.wast` grammar.
  - Script-related grammar rules include:
    - `script_module` supports:
      - `(module ...)` → `Textual`
      - `(module binary ...)` → `Encoded (name, bytestring)`
      - `(module quote ...)` → `Quoted (name, text)`
    - assertions: `assert_malformed`, `assert_invalid`, `assert_return`, `assert_trap`, etc.
    - meta commands: `(script ...)`, `(input ...)`, `(output ...)`.

- `script/script.ml`:
  - Defines the AST types for scripts: `definition`, `action`, `assertion`, `command`, `meta`, `script`.
  - This AST is **not part of the WebAssembly core standard**; it is test infrastructure.

- `text/parse.ml`:
  - Small wrapper turning Menhir parser entrypoints into `Parse.Script.parse`, `Parse.Module.parse`, etc.

- `script/run.ml`:
  - Implements the semantics of running scripts:
    - `run_definition`:
      - `Textual` → returns the module AST directly
      - `Encoded` → decodes binary bytes via `Decode.decode`
      - `Quoted` → parses module text via `Parse.Module.parse_string`
    - `run_command`:
      - `Module`: validate (`Valid.check_module`), then instantiate (`Import.link` + `Eval.init`) unless `-d`/`--dry`
      - `Register`: adds instance to a registry and wires it into the import resolver
      - `Action`: `Invoke` / `Get`
      - `Assertion`: checks expected failures / return values / trap messages
      - `Meta`: `(input ...)` loads another script; `(output ...)` triggers conversions

- `binary/decode.ml` + `binary/encode.ml`:
  - Binary codec used by `Encoded` modules and by conversions.

- `text/print.ml`:
  - Pretty-printer used for textual output and conversion stability checks.

- `script/js.ml`:
  - Converts scripts to JS for running in a JS engine.

- `host/spectest.ml`:
  - Implements the “spectest” host environment required by many spec tests.

Implication:
- To be faithful, a Rust port should mirror these **stages** (parse → run definitions → validate → instantiate → execute actions → check assertions), even if wasmly initially only implements a subset.

### 9.3 Why this matters for wasmly

`run.py` provides a **behavioral contract** for a test harness:

- batch execution
- expected failure encoding in filenames (`.fail.`)
- deterministic round-trip conversions (decode/encode stability)
- producing auxiliary artifacts for debugging (logs + generated conversions)

For wasmly, we likely **cannot** replicate everything immediately (e.g. JS emission), but we can keep the same shape:

- run each script
- generate and preserve intermediate artifacts
- perform round-trips where we have encoders/printers

### 9.4 Rust port plan for `run.py`

We can port `run.py` into a Rust test harness with two backends:

#### Backend 1: “External interpreter backend” (parity check)

Purpose: ensure our Rust harness matches the Python runner behavior.

- Spawn the spec interpreter (`wasm-spec/interpreter/wasm`) using `std::process::Command`.
- Reproduce the same pipeline as `run.py`:
  - `-d ... -o ...` conversions
  - run scripts
  - compare conversion outputs
  - support `--failfast`, `--out`, `--generate-js-only`

This backend is useful even before wasmly can run scripts.

#### Backend 2: “wasmly backend” (actual compliance testing)

Purpose: run the same suite against wasmly components.

Minimum viable (binary parser focused):
- For each script:
  - parse directives (Rust `wast` crate)
  - for every `(module binary ...)` and `(module ...)`:
    - produce `.wasm` bytes
    - run wasmly binary parser and assert expected outcome
- Map `.fail.` expectation either to:
  - “must fail at some directive” (coarse), or
  - a curated allowlist/denylist as we refine

Later phases:
- add validation mapping (`assert_invalid`, `assert_malformed`)
- add execution mapping (`assert_return`, `invoke`, etc.)
- add round-trip conversion checks once wasmly has:
  - binary writer, and/or
  - text printer (canonicalization)

#### `cargo test` integration shape

- Prefer `libtest-mimic` once we want *per-directive* tests.
- MVP can be `datatest-stable` with one test per `.wast` file.

Suggested CLI (optional, for dev convenience):

- `cargo run -p spec-harness -- --out target/spec-out --failfast wasm-spec/test/core/binary.wast`

### 9.5 Proposed crate layout for the port

Create `crates/spec-harness/` (dev tool, not the core library):

- `discover.rs` (glob `.wast` files, include simd)
- `pipeline.rs` (the run.py steps as a state machine)
- `backend/mod.rs`
  - `backend/spec_interp.rs` (spawns `wasm`)
  - `backend/wasmly.rs` (calls into wasmly)
- `artifacts.rs` (paths, cleanup, logging)
- `compare.rs` (file comparisons)
- `report.rs` (pretty failures)

---

## 10. Appendix: recommended initial target files

For binary parsing MVP:
- `wasm-spec/test/core/binary.wast`
- `wasm-spec/test/core/binary-leb128.wast`
- `wasm-spec/test/core/custom.wast`

For next phase (requires compiling text modules):
- `type.wast`, `imports.wast`, `exports.wast`, `nop.wast`
