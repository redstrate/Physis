// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{BinReaderExt, BinResult, binread};
use half::f16;
use std::ffi::CString;
use std::io::{Read, SeekFrom};

pub(crate) fn read_bool_from<T: std::convert::From<u8> + std::cmp::PartialEq>(x: T) -> bool {
    x == T::from(1u8)
}

pub(crate) fn write_bool_as<T: std::convert::From<u8>>(x: &bool) -> T {
    if *x { T::from(1u8) } else { T::from(0u8) }
}

/// Read a null-terminated UTF-8 string from a reader at its current position.
pub(crate) fn read_null_terminated_utf8<R: Read>(reader: &mut R) -> String {
    let mut bytes = Vec::new();
    let mut buf = [0u8; 1];
    while reader.read_exact(&mut buf).is_ok() && buf[0] != 0 {
        bytes.push(buf[0]);
    }
    String::from_utf8(bytes).unwrap_or_default()
}

/// Read a null-terminated UTF-8 string from a byte slice starting at `offset`.
/// Returns the decoded string and the offset immediately after the null terminator.
pub(crate) fn null_terminated_utf8(data: &[u8], offset: usize) -> (String, usize) {
    let end = data[offset..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| p + offset)
        .unwrap_or(data.len());
    let s = String::from_utf8(data[offset..end].to_vec()).unwrap_or_default();
    (s, end + 1)
}

pub(crate) fn read_string(byte_stream: Vec<u8>) -> String {
    let str = String::from_utf8(byte_stream).unwrap_or_default();
    str.trim_matches(char::from(0)).to_string() // trim \0 from the end of strings
}

pub(crate) fn write_string(str: &String) -> Vec<u8> {
    let c_string = CString::new(&**str).unwrap_or_default();
    c_string.as_bytes_with_nul().to_vec()
}

pub(crate) fn get_string_len(str: &String) -> usize {
    let c_string = CString::new(&**str).unwrap();
    c_string.count_bytes() + 1 // for the nul terminator
}

#[binrw::parser(reader)]
pub(crate) fn strings_parser(
    base_offset: u64,
    strings_offset: &Vec<u16>,
) -> BinResult<Vec<String>> {
    let mut strings: Vec<String> = vec![];

    for offset in strings_offset {
        reader.seek(SeekFrom::Start(base_offset + *offset as u64))?;
        strings.push(read_null_terminated_utf8(reader));
    }

    Ok(strings)
}

#[binrw::parser(reader)]
pub(crate) fn read_string_until_null() -> BinResult<String> {
    Ok(read_null_terminated_utf8(reader))
}

fn read_half1(data: [u16; 1]) -> Half1 {
    Half1 {
        value: f16::from_bits(data[0]),
    }
}

#[binread]
#[derive(Debug, Default, Clone, Copy)]
#[br(map = read_half1)]
pub(crate) struct Half1 {
    pub value: f16,
}

fn read_half2(data: [u16; 2]) -> Half2 {
    Half2 {
        x: f16::from_bits(data[0]),
        y: f16::from_bits(data[1]),
    }
}

#[binread]
#[derive(Debug, Default, Clone, Copy)]
#[br(map = read_half2)]
pub(crate) struct Half2 {
    pub x: f16,
    pub y: f16,
}

fn read_half3(data: [u16; 3]) -> Half3 {
    Half3 {
        r: f16::from_bits(data[0]),
        g: f16::from_bits(data[1]),
        b: f16::from_bits(data[2]),
    }
}

#[binread]
#[derive(Debug, Default, Clone, Copy)]
#[br(map = read_half3)]
pub(crate) struct Half3 {
    pub r: f16,
    pub g: f16,
    pub b: f16,
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

#[cfg(test)]
mod tests {
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
    fn get_string_len() {
        // Supposed to include the nul terminator
        assert_eq!(
            crate::common_file_operations::get_string_len(&"FOO".to_string()),
            4
        );
    }

    #[test]
    fn read_null_terminated_utf8_ascii() {
        let data = b"hello\0rest";
        let mut cursor = std::io::Cursor::new(&data[..]);
        assert_eq!(read_null_terminated_utf8(&mut cursor), "hello");
        // cursor should be positioned right after the null byte
        assert_eq!(cursor.position(), 6);
    }

    #[test]
    fn read_null_terminated_utf8_chinese() {
        // "你好" in UTF-8: [0xE4,0xBD,0xA0, 0xE5,0xA5,0xBD] + null
        let data = b"\xe4\xbd\xa0\xe5\xa5\xbd\0";
        let mut cursor = std::io::Cursor::new(&data[..]);
        assert_eq!(read_null_terminated_utf8(&mut cursor), "你好");
    }

    #[test]
    fn read_null_terminated_utf8_empty() {
        let data = b"\0trailing";
        let mut cursor = std::io::Cursor::new(&data[..]);
        assert_eq!(read_null_terminated_utf8(&mut cursor), "");
    }

    #[test]
    fn read_null_terminated_utf8_invalid_fallback() {
        // Invalid UTF-8 sequence: 0xFF is never valid in UTF-8
        let data: &[u8] = &[0xFF, 0xFE, 0x00];
        let mut cursor = std::io::Cursor::new(data);
        assert_eq!(read_null_terminated_utf8(&mut cursor), "");
    }

    #[test]
    fn null_terminated_utf8_ascii() {
        let data = b"foo\0bar\0";
        let (s, next) = null_terminated_utf8(data, 0);
        assert_eq!(s, "foo");
        assert_eq!(next, 4);
        let (s2, next2) = null_terminated_utf8(data, next);
        assert_eq!(s2, "bar");
        assert_eq!(next2, 8);
    }

    #[test]
    fn null_terminated_utf8_chinese() {
        // "装备" in UTF-8: [0xE8,0xA3,0x85, 0xE5,0xA4,0x87] + null
        let data = b"\xe8\xa3\x85\xe5\xa4\x87\0";
        let (s, _) = null_terminated_utf8(data, 0);
        assert_eq!(s, "装备");
    }

    #[test]
    fn null_terminated_utf8_at_offset() {
        let data = b"\0hello\0world\0";
        let (s, next) = null_terminated_utf8(data, 1);
        assert_eq!(s, "hello");
        assert_eq!(next, 7);
        let (s2, _) = null_terminated_utf8(data, next);
        assert_eq!(s2, "world");
    }

    #[test]
    fn null_terminated_utf8_empty_at_offset() {
        let data = b"a\0\0b\0";
        let (s, next) = null_terminated_utf8(data, 2);
        assert_eq!(s, "");
        assert_eq!(next, 3);
    }
}
