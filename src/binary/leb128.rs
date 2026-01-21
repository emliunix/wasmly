type Input<'a> = &'a [u8];

pub fn decode_u32(input: Input) -> nom::IResult<Input, u32> {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;
    let mut remaining = input;

    loop {
        if remaining.is_empty() {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Eof,
            }));
        }

        let byte = remaining[0];
        remaining = &remaining[1..];

        let value = byte & 0x7F;
        result |= (value as u32) << shift;

        if byte & 0x80 == 0 {
            return Ok((remaining, result));
        }

        shift += 7;
        if shift >= 32 {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::TooLarge,
            }));
        }
    }
}

pub fn decode_i32(input: Input) -> nom::IResult<Input, i32> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    let mut remaining = input;

    loop {
        if remaining.is_empty() {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::Eof,
            }));
        }

        let byte = remaining[0];
        remaining = &remaining[1..];

        let value = byte & 0x7F;
        result |= (value as u64) << shift;

        if byte & 0x80 == 0 {
            let sign_bit = byte & 0x40;
            if sign_bit != 0 {
                result |= (!0u64) << shift;
            }

            return Ok((remaining, result as i32));
        }

        shift += 7;
        if shift >= 32 {
            return Err(nom::Err::Error(nom::error::Error {
                input,
                code: nom::error::ErrorKind::TooLarge,
            }));
        }
    }
}

pub fn encode_u32(value: u32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut v = value;

    loop {
        let mut byte = (v & 0x7F) as u8;
        v >>= 7;

        if v == 0 {
            result.push(byte);
            break;
        } else {
            byte |= 0x80;
            result.push(byte);
        }
    }

    result
}

pub fn encode_i32(value: i32) -> Vec<u8> {
    let mut result = Vec::new();
    let mut v = value as u64;

    loop {
        let mut byte = (v & 0x7F) as u8;
        v >>= 7;

        if (v == 0 && (byte & 0x40) == 0) || (v == !0 && (byte & 0x40) != 0) {
            result.push(byte);
            break;
        } else {
            byte |= 0x80;
            result.push(byte);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_u32_zero() {
        let input = [0x00];
        let (remaining, value) = decode_u32(&input).unwrap();
        assert_eq!(value, 0);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_u32_small() {
        let input = [0x7F];
        let (remaining, value) = decode_u32(&input).unwrap();
        assert_eq!(value, 127);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_u32_two_bytes() {
        let input = [0x80, 0x01];
        let (remaining, value) = decode_u32(&input).unwrap();
        assert_eq!(value, 128);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_u32_three_bytes() {
        let input = [0x80, 0x81, 0x01];
        let (remaining, value) = decode_u32(&input).unwrap();
        assert_eq!(value, 16385);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_u32_max() {
        let input = [0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
        let (remaining, value) = decode_u32(&input).unwrap();
        assert_eq!(value, u32::MAX);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_i32_zero() {
        let input = [0x00];
        let (remaining, value) = decode_i32(&input).unwrap();
        assert_eq!(value, 0);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_i32_positive() {
        let input = [0x7F];
        let (remaining, value) = decode_i32(&input).unwrap();
        assert_eq!(value, 127);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_i32_negative() {
        let input = [0x7F, 0x7E];
        let (remaining, value) = decode_i32(&input).unwrap();
        assert_eq!(value, -2);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_decode_i32_large_negative() {
        let input = [0xFF, 0xFF, 0xFF, 0xFF, 0x07];
        let (remaining, value) = decode_i32(&input).unwrap();
        assert_eq!(value, i32::MIN);
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_encode_u32_zero() {
        let result = encode_u32(0);
        assert_eq!(result, [0x00]);
    }

    #[test]
    fn test_encode_u32_small() {
        let result = encode_u32(127);
        assert_eq!(result, [0x7F]);
    }

    #[test]
    fn test_encode_u32_two_bytes() {
        let result = encode_u32(128);
        assert_eq!(result, [0x80, 0x01]);
    }

    #[test]
    fn test_encode_u32_max() {
        let result = encode_u32(u32::MAX);
        assert_eq!(result, [0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    }

    #[test]
    fn test_encode_i32_zero() {
        let result = encode_i32(0);
        assert_eq!(result, [0x00]);
    }

    #[test]
    fn test_encode_i32_positive() {
        let result = encode_i32(127);
        assert_eq!(result, [0x7F]);
    }

    #[test]
    fn test_encode_i32_negative() {
        let result = encode_i32(-2);
        assert_eq!(result, [0x7E]);
    }

    #[test]
    fn test_roundtrip_u32_values() {
        let test_values = vec![0, 1, 127, 128, 255, 256, 65535, 65536, u32::MAX];

        for value in test_values {
            let encoded = encode_u32(value);
            let (remaining, decoded) = decode_u32(&encoded).unwrap();
            assert_eq!(value, decoded);
            assert_eq!(remaining.len(), 0);
        }
    }

    #[test]
    fn test_roundtrip_i32_values() {
        let test_values = vec![
            0,
            1,
            127,
            128,
            255,
            256,
            -1,
            -2,
            -127,
            -128,
            i32::MIN,
            i32::MAX,
        ];

        for value in test_values {
            let encoded = encode_i32(value);
            let (remaining, decoded) = decode_i32(&encoded).unwrap();
            assert_eq!(value, decoded);
            assert_eq!(remaining.len(), 0);
        }
    }

    #[test]
    fn test_decode_u32_incomplete() {
        let input = [0x80];
        let result = decode_u32(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_u32_too_large() {
        let input = [0x80, 0x80, 0x80, 0x80, 0x80, 0x80];
        let result = decode_u32(&input);
        assert!(result.is_err());
    }
}
