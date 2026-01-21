use crate::binary::error::{Located, ParseResult, SourceLocation};
use nom::bytes::complete::take;

type Input<'a> = &'a [u8];

pub fn parse_byte(input: Input) -> ParseResult<'_, Located<u8>> {
    let (remaining, bytes) = take(1usize)(input)?;
    let offset = 0;  // Parser starts at beginning of its input
    let location = SourceLocation::new(offset, 1);
    Ok((remaining, Located::new(bytes[0], location)))
}

pub fn parse_magic(input: Input) -> ParseResult<'_, Located<[u8; 4]>> {
    let base = input;
    let (remaining, bytes) = take(4usize)(input)?;
    let magic: [u8; 4] = bytes.try_into().unwrap();

    if &magic != &[0x00, 0x61, 0x73, 0x6D] {
        return Err(nom::Err::Error(nom::error::Error {
            input: base,
            code: nom::error::ErrorKind::Tag,
        }));
    }

    let offset = 0;  // Parser starts at beginning of its input
    let location = SourceLocation::new(offset, 4);
    Ok((remaining, Located::new(magic, location)))
}

pub fn parse_version(input: Input) -> ParseResult<'_, Located<[u8; 4]>> {
    let base = input;
    let (remaining, bytes) = take(4usize)(input)?;
    let version: [u8; 4] = bytes.try_into().unwrap();

    if &version != &[0x01, 0x00, 0x00, 0x00] {
        return Err(nom::Err::Error(nom::error::Error {
            input: base,
            code: nom::error::ErrorKind::Verify,
        }));
    }

    let offset = 0;  // Parser starts at beginning of its input
    let location = SourceLocation::new(offset, 4);
    Ok((remaining, Located::new(version, location)))
}

pub fn parse_section_header(input: Input) -> ParseResult<'_, (Located<u8>, Located<u32>)> {
    let (remaining, id) = parse_byte(input)?;
    let (remaining, length) = parse_leb128_u32(remaining)?;
    
    // Adjust location offset for length since it starts after the id byte
    let length_value = length.value;
    let length_len = length.location.length;
    let length_with_offset = Located::new(
        length_value,
        SourceLocation::new(1, length_len)
    );
    
    Ok((remaining, (id, length_with_offset)))
}

pub fn parse_name(input: Input) -> ParseResult<'_, Located<String>> {
    let base = input;
    let (remaining, length) = parse_leb128_u32(input)?;
    let length_byte_count = base.len() - remaining.len();
    let (rest, bytes) = take(length.into_inner() as usize)(remaining)?;

    match std::str::from_utf8(bytes) {
        Ok(s) => {
            // Offset is after the length bytes, length is the string length
            let offset = length_byte_count;
            let location = SourceLocation::new(offset, bytes.len());
            Ok((rest, Located::new(s.to_string(), location)))
        }
        Err(_) => Err(nom::Err::Error(nom::error::Error {
            input: base,
            code: nom::error::ErrorKind::Verify,
        })),
    }
}

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
            // Offset is 0 (starts at beginning), length is bytes consumed
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_byte_location() {
        let input = [0x42, 0xFF];
        let (remaining, byte) = parse_byte(&input).unwrap();
        assert_eq!(byte.value, 0x42);
        assert_eq!(byte.location.offset, 0);
        assert_eq!(byte.location.length, 1);
        assert_eq!(remaining, &[0xFFu8]);
    }

    #[test]
    fn test_parse_magic_location() {
        let input = [0x00, 0x61, 0x73, 0x6D, 0x01];
        let (remaining, magic) = parse_magic(&input).unwrap();
        assert_eq!(magic.value, [0x00, 0x61, 0x73, 0x6D]);
        assert_eq!(magic.location.offset, 0);
        assert_eq!(magic.location.length, 4);
        assert_eq!(remaining, &[0x01u8]);
    }

    #[test]
    fn test_parse_version_location() {
        let input = [0x01, 0x00, 0x00, 0x00, 0xFF];
        let (remaining, version) = parse_version(&input).unwrap();
        assert_eq!(version.value, [0x01, 0x00, 0x00, 0x00]);
        assert_eq!(version.location.offset, 0);
        assert_eq!(version.location.length, 4);
        assert_eq!(remaining, &[0xFFu8]);
    }

    #[test]
    fn test_parse_leb128_u32_small_location() {
        let input = [0x7F, 0xFF];
        let (remaining, value) = parse_leb128_u32(&input).unwrap();
        assert_eq!(value.value, 127);
        assert_eq!(value.location.offset, 0);
        assert_eq!(value.location.length, 1);
        assert_eq!(remaining, &[0xFFu8]);
    }

    #[test]
    fn test_parse_leb128_u32_medium_location() {
        let input = [0x80, 0x01, 0xFF];
        let (remaining, value) = parse_leb128_u32(&input).unwrap();
        assert_eq!(value.value, 128);
        assert_eq!(value.location.offset, 0);
        assert_eq!(value.location.length, 2);
        assert_eq!(remaining, &[0xFFu8]);
    }

    #[test]
    fn test_parse_leb128_u32_large_location() {
        let input = [0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0xFF];
        let (remaining, value) = parse_leb128_u32(&input).unwrap();
        assert_eq!(value.value, u32::MAX);
        assert_eq!(value.location.offset, 0);
        assert_eq!(value.location.length, 5);
        assert_eq!(remaining, &[0xFFu8]);
    }

    #[test]
    fn test_parse_name_location() {
        let input = [0x03, 0x41, 0x42, 0x43, 0xFF];
        let (remaining, name) = parse_name(&input).unwrap();
        assert_eq!(name.value, "ABC");
        assert_eq!(name.location.offset, 1);
        assert_eq!(name.location.length, 3);
        assert_eq!(remaining, &[0xFFu8]);
    }

    #[test]
    fn test_parse_section_header_location() {
        let input = [0x01, 0x02, 0xFF];
        let (remaining, (id, length)) = parse_section_header(&input).unwrap();
        assert_eq!(id.value, 0x01);
        assert_eq!(id.location.offset, 0);
        assert_eq!(length.value, 2);
        assert_eq!(length.location.offset, 1);
        assert_eq!(remaining, &[0xFFu8]);
    }
}
