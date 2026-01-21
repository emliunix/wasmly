use crate::binary::error::{Located, ParseResult};
use crate::binary::primitives::parse_byte;
use crate::module::{Code, Expr};
use crate::types::{FuncType, Instr, Limits, MemType, RefType, TableType, ValType};
use nom::bytes::complete::take;

type Input<'a> = &'a [u8];

// ============================================================================
// Vector Parser (common to all sections)
// ============================================================================

/// Parse a vector: length:u32 followed by that many elements
pub fn parse_vec<'a, T, F>(input: Input<'a>, parser: F) -> ParseResult<'a, Vec<T>>
where
    F: Fn(Input<'a>) -> ParseResult<'a, T>,
{
    let (mut remaining, length) = parse_leb128_u32(input)?;
    let mut elements = Vec::with_capacity(length.value as usize);

    for _ in 0..length.value {
        let (rest, element) = parser(remaining)?;
        elements.push(element);
        remaining = rest;
    }

    Ok((remaining, elements))
}

// ============================================================================
// Value Types
// ============================================================================

pub fn parse_valtype(input: Input) -> ParseResult<'_, ValType> {
    let (remaining, byte) = parse_byte(input)?;

    let valtype = match byte.value {
        0x7F => ValType::I32,
        0x7E => ValType::I64,
        0x7D => ValType::F32,
        0x7C => ValType::F64,
        0x7B => ValType::V128,
        0x70 => ValType::FuncRef,
        0x6F => ValType::ExternRef,
        _ => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Tag,
            }))
        }
    };

    Ok((remaining, valtype))
}

pub fn parse_reftype(input: Input) -> ParseResult<'_, RefType> {
    let (remaining, byte) = parse_byte(input)?;

    let reftype = match byte.value {
        0x70 => RefType::FuncRef,
        0x6F => RefType::ExternRef,
        _ => {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Tag,
            }))
        }
    };

    Ok((remaining, reftype))
}

// ============================================================================
// Section 1: Type Section
// ============================================================================

/// Parse a function type: 0x60 params:vec(valtype) results:vec(valtype)
pub fn parse_functype(input: Input) -> ParseResult<'_, FuncType> {
    let base = input;
    let (remaining, byte) = parse_byte(input)?;

    if byte.value != 0x60 {
        return Err(nom::Err::Error(nom::error::Error {
            input: base,
            code: nom::error::ErrorKind::Tag,
        }));
    }

    let (remaining, params) = parse_vec(remaining, parse_valtype)?;
    let (remaining, results) = parse_vec(remaining, parse_valtype)?;

    Ok((remaining, FuncType { params, results }))
}

/// Parse the type section: vec(functype)
pub fn parse_type_section(input: Input) -> ParseResult<'_, Vec<FuncType>> {
    parse_vec(input, parse_functype)
}

// ============================================================================
// Section 3: Function Section
// ============================================================================

/// Parse the function section: vec(typeidx)
pub fn parse_function_section(input: Input) -> ParseResult<'_, Vec<u32>> {
    parse_vec(input, |input| {
        let (remaining, typeidx) = parse_leb128_u32(input)?;
        Ok((remaining, typeidx.value))
    })
}

// ============================================================================
// Section 4: Table Section
// ============================================================================

/// Parse limits: flags:u8 min:u32 [max:u32]
pub fn parse_limits(input: Input) -> ParseResult<'_, Limits> {
    let (remaining, flags) = parse_byte(input)?;
    let (remaining, min) = parse_leb128_u32(remaining)?;

    match flags.value {
        0x00 => Ok((
            remaining,
            Limits {
                min: min.value,
                max: None,
            },
        )),
        0x01 => {
            let (remaining, max) = parse_leb128_u32(remaining)?;
            Ok((
                remaining,
                Limits {
                    min: min.value,
                    max: Some(max.value),
                },
            ))
        }
        _ => Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Tag,
        })),
    }
}

/// Parse table type: reftype limits
pub fn parse_tabletype(input: Input) -> ParseResult<'_, TableType> {
    let (remaining, elem_type) = parse_reftype(input)?;
    let (remaining, limits) = parse_limits(remaining)?;
    Ok((remaining, TableType { limits, elem_type }))
}

// ============================================================================
// Section 5: Memory Section
// ============================================================================

/// Parse memory type: limits
pub fn parse_memtype(input: Input) -> ParseResult<'_, MemType> {
    let (remaining, limits) = parse_limits(input)?;
    Ok((remaining, MemType { limits }))
}

// ============================================================================
// Section 10: Code Section
// ============================================================================

/// Parse a locals declaration: count:u32 valtype
pub fn parse_locals(input: Input) -> ParseResult<'_, Vec<ValType>> {
    let (remaining, count) = parse_leb128_u32(input)?;
    let (remaining, valtype) = parse_valtype(remaining)?;

    // Expand count into a vec of repeated valtypes
    let locals = vec![valtype; count.value as usize];
    Ok((remaining, locals))
}

/// Parse instructions until 0x0B (end) is reached
/// For now, we'll implement a subset of instructions to get started
pub fn parse_instructions(input: Input) -> ParseResult<'_, Vec<Instr>> {
    let mut remaining = input;
    let mut instrs = Vec::new();

    loop {
        let (rest, opcode) = parse_byte(remaining)?;
        remaining = rest;

        match opcode.value {
            0x0B => break, // end
            0x01 => {
                // nop
                instrs.push(Instr::Nop);
            }
            0x1A => {
                // drop
                instrs.push(Instr::Drop);
            }
            0x20 => {
                // local.get
                let (rest, idx) = parse_leb128_u32(remaining)?;
                remaining = rest;
                instrs.push(Instr::LocalGet(idx.value as usize));
            }
            0x21 => {
                // local.set
                let (rest, idx) = parse_leb128_u32(remaining)?;
                remaining = rest;
                instrs.push(Instr::LocalSet(idx.value as usize));
            }
            0x41 => {
                // i32.const
                let (rest, value) = parse_leb128_i32(remaining)?;
                remaining = rest;
                instrs.push(Instr::I32Const(value.value));
            }
            0x6A => {
                // i32.add
                instrs.push(Instr::I32Add);
            }
            0x6B => {
                // i32.sub
                instrs.push(Instr::I32Sub);
            }
            0x6C => {
                // i32.mul
                instrs.push(Instr::I32Mul);
            }
            _ => {
                // Unsupported opcode for now
                return Err(nom::Err::Error(nom::error::Error {
                    input,
                    code: nom::error::ErrorKind::Tag,
                }));
            }
        }
    }

    Ok((remaining, instrs))
}

/// Parse an expression (instructions ending with 0x0B)
pub fn parse_expr(input: Input) -> ParseResult<'_, Expr> {
    let (remaining, instrs) = parse_instructions(input)?;
    Ok((remaining, Expr { instrs }))
}

/// Parse a code entry: size:u32 code
pub fn parse_code(input: Input) -> ParseResult<'_, Code> {
    let (remaining, size) = parse_leb128_u32(input)?;
    let size_val = size.value as usize;

    // Take exactly 'size' bytes for the code body
    let (rest, code_bytes) = take(size_val)(remaining)?;

    // Parse the code body starting from the beginning of code_bytes
    let (after_locals, locals_vec) = parse_vec(code_bytes, parse_locals)?;

    // Flatten the vec of vecs into a single vec
    let locals: Vec<ValType> = locals_vec.into_iter().flatten().collect();

    // Parse body from after the locals to the end
    let (_, body) = parse_expr(after_locals)?;

    Ok((rest, Code { locals, body }))
}

/// Parse the code section: vec(code)
pub fn parse_code_section(input: Input) -> ParseResult<'_, Vec<Code>> {
    parse_vec(input, parse_code)
}

// ============================================================================
// LEB128 Parser (helpers)
// ============================================================================

fn parse_leb128_u32(input: Input) -> ParseResult<'_, Located<u32>> {
    let base = input;
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    let mut remaining = input;
    let mut bytes_consumed = 0;

    loop {
        let (rest, byte) = take(1usize)(remaining)?;
        remaining = rest;
        bytes_consumed += 1;

        let value = byte[0] & 0x7F;
        result |= (value as u32) << shift;

        if byte[0] & 0x80 == 0 {
            use crate::binary::error::SourceLocation;
            let offset = 0;
            let location = SourceLocation::new(offset, bytes_consumed);
            return Ok((remaining, Located::new(result, location)));
        }

        shift += 7;
        if shift >= 32 {
            return Err(nom::Err::Error(nom::error::Error {
                input: base,
                code: nom::error::ErrorKind::TooLarge,
            }));
        }
    }
}

fn parse_leb128_i32(input: Input) -> ParseResult<'_, Located<i32>> {
    let base = input;
    let mut result: i32 = 0;
    let mut shift: u32 = 0;
    let mut remaining = input;
    let mut bytes_consumed = 0;

    loop {
        let (rest, byte) = take(1usize)(remaining)?;
        remaining = rest;
        bytes_consumed += 1;

        let value = (byte[0] & 0x7F) as i32;
        result |= value << shift;

        if byte[0] & 0x80 == 0 {
            // Sign extend if necessary
            if shift < 32 && (byte[0] & 0x40) != 0 {
                result |= !0 << (shift + 7);
            }

            use crate::binary::error::SourceLocation;
            let offset = 0;
            let location = SourceLocation::new(offset, bytes_consumed);
            return Ok((remaining, Located::new(result, location)));
        }

        shift += 7;
        if shift >= 32 {
            return Err(nom::Err::Error(nom::error::Error {
                input: base,
                code: nom::error::ErrorKind::TooLarge,
            }));
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_valtype_i32() {
        let input = [0x7F, 0xFF];
        let (remaining, valtype) = parse_valtype(&input).unwrap();
        assert_eq!(valtype, ValType::I32);
        assert_eq!(remaining, &[0xFFu8]);
    }

    #[test]
    fn test_parse_valtype_i64() {
        let input = [0x7E];
        let (_, valtype) = parse_valtype(&input).unwrap();
        assert_eq!(valtype, ValType::I64);
    }

    #[test]
    fn test_parse_valtype_f32() {
        let input = [0x7D];
        let (_, valtype) = parse_valtype(&input).unwrap();
        assert_eq!(valtype, ValType::F32);
    }

    #[test]
    fn test_parse_valtype_f64() {
        let input = [0x7C];
        let (_, valtype) = parse_valtype(&input).unwrap();
        assert_eq!(valtype, ValType::F64);
    }

    #[test]
    fn test_parse_valtype_funcref() {
        let input = [0x70];
        let (_, valtype) = parse_valtype(&input).unwrap();
        assert_eq!(valtype, ValType::FuncRef);
    }

    #[test]
    fn test_parse_valtype_externref() {
        let input = [0x6F];
        let (_, valtype) = parse_valtype(&input).unwrap();
        assert_eq!(valtype, ValType::ExternRef);
    }

    #[test]
    fn test_parse_functype_empty() {
        // 0x60 (functype tag), 0x00 (0 params), 0x00 (0 results)
        let input = [0x60, 0x00, 0x00];
        let (remaining, functype) = parse_functype(&input).unwrap();
        assert_eq!(functype.params.len(), 0);
        assert_eq!(functype.results.len(), 0);
        assert_eq!(remaining, &[]);
    }

    #[test]
    fn test_parse_functype_single_param() {
        // 0x60, 0x01 (1 param), 0x7F (i32), 0x00 (0 results)
        let input = [0x60, 0x01, 0x7F, 0x00];
        let (_, functype) = parse_functype(&input).unwrap();
        assert_eq!(functype.params.len(), 1);
        assert_eq!(functype.params[0], ValType::I32);
        assert_eq!(functype.results.len(), 0);
    }

    #[test]
    fn test_parse_functype_single_result() {
        // 0x60, 0x00 (0 params), 0x01 (1 result), 0x7F (i32)
        let input = [0x60, 0x00, 0x01, 0x7F];
        let (_, functype) = parse_functype(&input).unwrap();
        assert_eq!(functype.params.len(), 0);
        assert_eq!(functype.results.len(), 1);
        assert_eq!(functype.results[0], ValType::I32);
    }

    #[test]
    fn test_parse_functype_multiple_params() {
        // 0x60, 0x02 (2 params), 0x7F 0x7E (i32, i64), 0x00 (0 results)
        let input = [0x60, 0x02, 0x7F, 0x7E, 0x00];
        let (_, functype) = parse_functype(&input).unwrap();
        assert_eq!(functype.params.len(), 2);
        assert_eq!(functype.params[0], ValType::I32);
        assert_eq!(functype.params[1], ValType::I64);
        assert_eq!(functype.results.len(), 0);
    }

    #[test]
    fn test_parse_functype_multiple_results() {
        // 0x60, 0x00 (0 params), 0x02 (2 results), 0x7D 0x7C (f32, f64)
        let input = [0x60, 0x00, 0x02, 0x7D, 0x7C];
        let (_, functype) = parse_functype(&input).unwrap();
        assert_eq!(functype.params.len(), 0);
        assert_eq!(functype.results.len(), 2);
        assert_eq!(functype.results[0], ValType::F32);
        assert_eq!(functype.results[1], ValType::F64);
    }

    #[test]
    fn test_parse_functype_complex() {
        // 0x60, 0x04 (4 params), 0x7F 0x7E 0x7D 0x7C, 0x02 (2 results), 0x7F 0x7E
        let input = [0x60, 0x04, 0x7F, 0x7E, 0x7D, 0x7C, 0x02, 0x7F, 0x7E];
        let (_, functype) = parse_functype(&input).unwrap();
        assert_eq!(functype.params.len(), 4);
        assert_eq!(functype.params[0], ValType::I32);
        assert_eq!(functype.params[1], ValType::I64);
        assert_eq!(functype.params[2], ValType::F32);
        assert_eq!(functype.params[3], ValType::F64);
        assert_eq!(functype.results.len(), 2);
        assert_eq!(functype.results[0], ValType::I32);
        assert_eq!(functype.results[1], ValType::I64);
    }

    #[test]
    fn test_parse_type_section_empty() {
        // 0x00 (0 types)
        let input = [0x00];
        let (_, types) = parse_type_section(&input).unwrap();
        assert_eq!(types.len(), 0);
    }

    #[test]
    fn test_parse_type_section_single() {
        // 0x01 (1 type), 0x60 0x00 0x00 (empty functype)
        let input = [0x01, 0x60, 0x00, 0x00];
        let (_, types) = parse_type_section(&input).unwrap();
        assert_eq!(types.len(), 1);
        assert_eq!(types[0].params.len(), 0);
        assert_eq!(types[0].results.len(), 0);
    }

    #[test]
    fn test_parse_type_section_multiple() {
        // 0x02 (2 types)
        // Type 0: 0x60 0x00 0x01 0x7F ([] -> [i32])
        // Type 1: 0x60 0x01 0x7F 0x00 ([i32] -> [])
        let input = [0x02, 0x60, 0x00, 0x01, 0x7F, 0x60, 0x01, 0x7F, 0x00];
        let (_, types) = parse_type_section(&input).unwrap();
        assert_eq!(types.len(), 2);

        // Type 0: [] -> [i32]
        assert_eq!(types[0].params.len(), 0);
        assert_eq!(types[0].results.len(), 1);
        assert_eq!(types[0].results[0], ValType::I32);

        // Type 1: [i32] -> []
        assert_eq!(types[1].params.len(), 1);
        assert_eq!(types[1].params[0], ValType::I32);
        assert_eq!(types[1].results.len(), 0);
    }

    #[test]
    fn test_parse_limits_no_max() {
        // 0x00 (no max), 0x0A (min = 10)
        let input = [0x00, 0x0A];
        let (_, limits) = parse_limits(&input).unwrap();
        assert_eq!(limits.min, 10);
        assert_eq!(limits.max, None);
    }

    #[test]
    fn test_parse_limits_with_max() {
        // 0x01 (has max), 0x0A (min = 10), 0x14 (max = 20)
        let input = [0x01, 0x0A, 0x14];
        let (_, limits) = parse_limits(&input).unwrap();
        assert_eq!(limits.min, 10);
        assert_eq!(limits.max, Some(20));
    }

    #[test]
    fn test_parse_tabletype() {
        // 0x70 (funcref), 0x00 (no max), 0x0A (min = 10)
        let input = [0x70, 0x00, 0x0A];
        let (_, tabletype) = parse_tabletype(&input).unwrap();
        assert_eq!(tabletype.elem_type, RefType::FuncRef);
        assert_eq!(tabletype.limits.min, 10);
        assert_eq!(tabletype.limits.max, None);
    }

    #[test]
    fn test_parse_memtype() {
        // 0x00 (no max), 0x01 (min = 1 page)
        let input = [0x00, 0x01];
        let (_, memtype) = parse_memtype(&input).unwrap();
        assert_eq!(memtype.limits.min, 1);
        assert_eq!(memtype.limits.max, None);
    }

    // Function Section Tests

    #[test]
    fn test_parse_function_section_empty() {
        // 0x00 (0 functions)
        let input = [0x00];
        let (_, functions) = parse_function_section(&input).unwrap();
        assert_eq!(functions.len(), 0);
    }

    #[test]
    fn test_parse_function_section_single() {
        // 0x01 (1 function), 0x00 (type index 0)
        let input = [0x01, 0x00];
        let (_, functions) = parse_function_section(&input).unwrap();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0], 0);
    }

    #[test]
    fn test_parse_function_section_multiple() {
        // 0x03 (3 functions), 0x00 0x01 0x00 (type indices)
        let input = [0x03, 0x00, 0x01, 0x00];
        let (_, functions) = parse_function_section(&input).unwrap();
        assert_eq!(functions.len(), 3);
        assert_eq!(functions[0], 0);
        assert_eq!(functions[1], 1);
        assert_eq!(functions[2], 0);
    }

    // Code Section Tests

    #[test]
    fn test_parse_instructions_nop() {
        // 0x01 (nop), 0x0B (end)
        let input = [0x01, 0x0B];
        let (_, instrs) = parse_instructions(&input).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0], Instr::Nop));
    }

    #[test]
    fn test_parse_instructions_i32_const() {
        // 0x41 (i32.const), 0x2A (42), 0x0B (end)
        let input = [0x41, 0x2A, 0x0B];
        let (_, instrs) = parse_instructions(&input).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0], Instr::I32Const(42)));
    }

    #[test]
    fn test_parse_instructions_i32_add() {
        // 0x41 0x01 (i32.const 1)
        // 0x41 0x02 (i32.const 2)
        // 0x6A (i32.add)
        // 0x0B (end)
        let input = [0x41, 0x01, 0x41, 0x02, 0x6A, 0x0B];
        let (_, instrs) = parse_instructions(&input).unwrap();
        assert_eq!(instrs.len(), 3);
        assert!(matches!(instrs[0], Instr::I32Const(1)));
        assert!(matches!(instrs[1], Instr::I32Const(2)));
        assert!(matches!(instrs[2], Instr::I32Add));
    }

    #[test]
    fn test_parse_instructions_local_get() {
        // 0x20 (local.get), 0x00 (local index 0), 0x0B (end)
        let input = [0x20, 0x00, 0x0B];
        let (_, instrs) = parse_instructions(&input).unwrap();
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0], Instr::LocalGet(0)));
    }

    #[test]
    fn test_parse_locals() {
        // 0x02 (count = 2), 0x7F (i32)
        let input = [0x02, 0x7F];
        let (_, locals) = parse_locals(&input).unwrap();
        assert_eq!(locals.len(), 2);
        assert_eq!(locals[0], ValType::I32);
        assert_eq!(locals[1], ValType::I32);
    }

    #[test]
    fn test_parse_code_empty_function() {
        // size: 0x02 (2 bytes)
        // locals: 0x00 (no locals)
        // body: 0x0B (end)
        let input = [0x02, 0x00, 0x0B];
        let (_, code) = parse_code(&input).unwrap();
        assert_eq!(code.locals.len(), 0);
        assert_eq!(code.body.instrs.len(), 0);
    }

    #[test]
    fn test_parse_code_with_locals() {
        // size: 0x04 (4 bytes)
        // locals: 0x01 (1 group), 0x02 (count = 2), 0x7F (i32)
        // body: 0x0B (end)
        let input = [0x04, 0x01, 0x02, 0x7F, 0x0B];
        let (_, code) = parse_code(&input).unwrap();
        assert_eq!(code.locals.len(), 2);
        assert_eq!(code.locals[0], ValType::I32);
        assert_eq!(code.locals[1], ValType::I32);
    }

    #[test]
    fn test_parse_code_with_instructions() {
        // size: 0x04 (4 bytes)
        // locals: 0x00 (no locals)
        // body: 0x41 0x2A (i32.const 42), 0x0B (end)
        let input = [0x04, 0x00, 0x41, 0x2A, 0x0B];
        let (_, code) = parse_code(&input).unwrap();
        assert_eq!(code.locals.len(), 0);
        assert_eq!(code.body.instrs.len(), 1);
        assert!(matches!(code.body.instrs[0], Instr::I32Const(42)));
    }

    #[test]
    fn test_parse_code_section_empty() {
        // 0x00 (0 code entries)
        let input = [0x00];
        let (_, codes) = parse_code_section(&input).unwrap();
        assert_eq!(codes.len(), 0);
    }

    #[test]
    fn test_parse_code_section_single() {
        // 0x01 (1 code entry)
        // size: 0x02, locals: 0x00, body: 0x0B
        let input = [0x01, 0x02, 0x00, 0x0B];
        let (_, codes) = parse_code_section(&input).unwrap();
        assert_eq!(codes.len(), 1);
        assert_eq!(codes[0].locals.len(), 0);
    }
}
