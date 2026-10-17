// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{BinRead, BinReaderExt, BinResult};
use half::f16;
use std::ffi::CString;

pub(crate) fn read_bool_from<T: std::convert::From<u8> + std::cmp::PartialEq>(x: T) -> bool {
    x != T::from(0u8)
}

pub(crate) fn write_bool_as<T: std::convert::From<u8>>(x: &bool) -> T {
    if *x { T::from(1u8) } else { T::from(0u8) }
}

/// Read a null-terminated UTF-8 string from a reader at its current position.
#[binrw::parser(reader)]
pub(crate) fn read_string_until_nul() -> BinResult<String> {
    let mut bytes = Vec::new();
    while let Ok(byte) = u8::read_ne(reader)
        && byte != 0
    {
        bytes.push(byte);
    }
    Ok(read_string(bytes))
}

/// Read a null-terminated UTF-8 string from a byte slice starting at `offset`.
/// Returns the decoded string and the offset immediately after the null terminator.
pub(crate) fn read_string_from_offset(data: &[u8], offset: usize) -> (String, usize) {
    if offset > data.len() {
        return (String::default(), 0);
    }

    let end = data[offset..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| p + offset)
        .unwrap_or(data.len());
    let s = String::from_utf8(data[offset..end].to_vec()).unwrap_or_default();
    (s, end + 1)
}

pub(crate) fn read_string(mut byte_stream: Vec<u8>) -> String {
    // Ensure we don't include the nul terminator
    let end = byte_stream
        .iter()
        .position(|b| *b == 0)
        .unwrap_or(byte_stream.len());
    byte_stream.truncate(end);
    String::from_utf8(byte_stream).unwrap_or_default()
}

pub(crate) fn write_string(str: &String) -> Vec<u8> {
    let c_string = CString::new(&**str).unwrap_or_default();
    c_string.as_bytes_with_nul().to_vec()
}

pub(crate) fn read_half1(data: u16) -> f32 {
    f16::from_bits(data).to_f32_const()
}

pub(crate) fn write_half1(data: &f32) -> [u8; 2] {
    let data0 = data.to_le_bytes();

    [data0[0], data0[1]]
}

pub(crate) fn read_half2(data: [u16; 2]) -> [f32; 2] {
    [
        f16::from_bits(data[0]).to_f32_const(),
        f16::from_bits(data[1]).to_f32_const(),
    ]
}

pub(crate) fn write_half2(data: &[f32; 2]) -> [u8; 4] {
    let data0 = data[0].to_le_bytes();
    let data1 = data[1].to_le_bytes();

    [data0[0], data0[1], data1[0], data1[1]]
}

pub(crate) fn read_half3(data: [u16; 3]) -> [f32; 3] {
    [
        f16::from_bits(data[0]).to_f32_const(),
        f16::from_bits(data[1]).to_f32_const(),
        f16::from_bits(data[2]).to_f32_const(),
    ]
}

pub(crate) fn write_half3(data: &[f32; 3]) -> [u8; 6] {
    let data0 = data[0].to_le_bytes();
    let data1 = data[1].to_le_bytes();
    let data2 = data[2].to_le_bytes();

    [data0[0], data0[1], data1[0], data1[1], data2[0], data2[1]]
}

pub(crate) fn read_half4(data: [u16; 4]) -> [f32; 4] {
    [
        f16::from_bits(data[0]).to_f32_const(),
        f16::from_bits(data[1]).to_f32_const(),
        f16::from_bits(data[2]).to_f32_const(),
        f16::from_bits(data[3]).to_f32_const(),
    ]
}

pub(crate) fn write_half4(data: &[f32; 4]) -> [u8; 8] {
    let data0 = data[0].to_le_bytes();
    let data1 = data[1].to_le_bytes();
    let data2 = data[2].to_le_bytes();
    let data3 = data[3].to_le_bytes();

    [
        data0[0], data0[1], data1[0], data1[1], data2[0], data2[1], data3[0], data3[1],
    ]
}

/// Reads a 4 byte string.
#[binrw::parser(reader, endian)]
pub(crate) fn read_short_identifier() -> BinResult<String> {
    let bytes = reader.read_type_args::<[u8; 4]>(endian, ())?.to_vec();

    Ok(String::from_utf8(bytes)
        .map_err(|orig_err| binrw::Error::Custom {
            pos: 0,
            err: Box::new(orig_err),
        })?
        .trim_matches(char::from(0))
        .to_string())
}

/// Writes a 4 byte string.
#[binrw::writer(writer)]
pub fn write_short_identifier(identifier: &String) -> BinResult<()> {
    let mut bytes = identifier.as_bytes().to_vec();
    bytes.resize(4, 0); // Pad to 4 bytes
    writer.write_all(&bytes)?;

    Ok(())
}

/// Used by the game's formats to denote something as needing Dawntrail & above behavior.
pub(crate) const DAWNTRAIL_MARKER: &[u8; 4] = b"007V";

pub(crate) fn read_dawntrail_marker(x: [u8; 4]) -> bool {
    &x == DAWNTRAIL_MARKER
}

pub(crate) fn write_dawntrail_marker(x: &bool) -> [u8; 4] {
    if *x { *DAWNTRAIL_MARKER } else { [0; 4] }
}

#[cfg(test)]
mod tests {
    use binrw::Endian;

    use super::*;

    const DATA: [u8; 2] = [0u8, 1u8];

    // TODO: add tests for u16

    #[test]
    fn read_bool_u8() {
        assert!(!read_bool_from::<u8>(DATA[0]));
        assert!(read_bool_from::<u8>(DATA[1]));
    }

    #[test]
    fn write_bool_u8() {
        assert_eq!(write_bool_as::<u8>(&false), DATA[0]);
        assert_eq!(write_bool_as::<u8>(&true), DATA[1]);
    }

    // "FOO\0"
    const STRING_DATA: [u8; 4] = [0x46u8, 0x4Fu8, 0x4Fu8, 0x0u8];

    #[test]
    fn read_string() {
        // The nul terminator is supposed to be removed
        assert_eq!(
            crate::common_file_operations::read_string(STRING_DATA.to_vec()),
            "FOO".to_string()
        );
    }

    #[test]
    fn write_string() {
        // Supposed to include the nul terminator
        assert_eq!(
            crate::common_file_operations::write_string(&"FOO".to_string()),
            STRING_DATA.to_vec()
        );
    }

    #[test]
    fn read_null_terminated_utf8_ascii() {
        let data = b"hello\0rest";
        let mut cursor = std::io::Cursor::new(&data[..]);
        assert_eq!(
            read_string_until_nul(&mut cursor, Endian::Little, ()).unwrap(),
            "hello"
        );
        // cursor should be positioned right after the null byte
        assert_eq!(cursor.position(), 6);
    }

    #[test]
    fn read_null_terminated_utf8_chinese() {
        // "你好" in UTF-8: [0xE4,0xBD,0xA0, 0xE5,0xA5,0xBD] + null
        let data = b"\xe4\xbd\xa0\xe5\xa5\xbd\0";
        let mut cursor = std::io::Cursor::new(&data[..]);
        assert_eq!(
            read_string_until_nul(&mut cursor, Endian::Little, ()).unwrap(),
            "你好"
        );
    }

    #[test]
    fn read_null_terminated_utf8_empty() {
        let data = b"\0trailing";
        let mut cursor = std::io::Cursor::new(&data[..]);
        assert_eq!(
            read_string_until_nul(&mut cursor, Endian::Little, ()).unwrap(),
            ""
        );
    }

    #[test]
    fn read_null_terminated_utf8_invalid_fallback() {
        // Invalid UTF-8 sequence: 0xFF is never valid in UTF-8
        let data: &[u8] = &[0xFF, 0xFE, 0x00];
        let mut cursor = std::io::Cursor::new(data);
        assert_eq!(
            read_string_until_nul(&mut cursor, Endian::Little, ()).unwrap(),
            ""
        );
    }

    #[test]
    fn null_terminated_utf8_ascii() {
        let data = b"foo\0bar\0";
        let (s, next) = read_string_from_offset(data, 0);
        assert_eq!(s, "foo");
        assert_eq!(next, 4);
        let (s2, next2) = read_string_from_offset(data, next);
        assert_eq!(s2, "bar");
        assert_eq!(next2, 8);
    }

    #[test]
    fn null_terminated_utf8_chinese() {
        // "装备" in UTF-8: [0xE8,0xA3,0x85, 0xE5,0xA4,0x87] + null
        let data = b"\xe8\xa3\x85\xe5\xa4\x87\0";
        let (s, _) = read_string_from_offset(data, 0);
        assert_eq!(s, "装备");
    }

    #[test]
    fn null_terminated_utf8_at_offset() {
        let data = b"\0hello\0world\0";
        let (s, next) = read_string_from_offset(data, 1);
        assert_eq!(s, "hello");
        assert_eq!(next, 7);
        let (s2, _) = read_string_from_offset(data, next);
        assert_eq!(s2, "world");
    }

    #[test]
    fn null_terminated_utf8_empty_at_offset() {
        let data = b"a\0\0b\0";
        let (s, next) = read_string_from_offset(data, 2);
        assert_eq!(s, "");
        assert_eq!(next, 3);
    }
}
