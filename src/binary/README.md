# WASM Binary Parser

This module provides a WebAssembly binary format parser for wasmly.rs.

## Overview

The parser uses the [nom](https://docs.rs/nom/) parser combinator library for robust, composable binary parsing with accurate source location tracking.

## Features

- LEB128 encoding/decoding for all integers (unsigned and signed)
- Section-based parsing (type, import, function, code, export, data)
- Instruction parsing with full WASM 1.0 specification support
- Accurate error messages with byte offset locations
- Integration with existing `Instr` and `Ty` types

## Usage

```rust
use wasmly::binary::Module;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: wasmly-parser <input.wasm>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let bytes = std::fs::read(filename)?;

    match wasmly::binary::parse_module(&bytes) {
        Ok(module) => {
            println!("Parsed module:");
            println!("  Types: {}", module.types.len());
            println!("  Functions: {}", module.functions.len());
            println!("  Imports: {}", module.imports.len());
            println!("  Exports: {}", module.exports.len());
        }
        Err(e) => {
            eprintln!("Parse error at byte {}: {}", e.position);
            return Err(Box::new(e));
        }
    }
}
```

## Implementation Status

- [x] LEB128 utilities - decode/encode signed/unsigned integers
- [x] Binary primitives - magic, version, sections, names
- [ ] Section parsers - type, import, function, export, data sections
- [ ] Instruction parser - map binary encodings to `Instr` enum

## Documentation

See `docs/parser.md` for detailed implementation plan and binary parsing conventions.
