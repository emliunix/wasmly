use crate::binary::error::{Located, ParseResult};
use crate::binary::primitives::parse_byte;
use crate::types::{FuncType, Limits, MemType, RefType, TableType, ValType};
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
// LEB128 Parser (helper)
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
}
