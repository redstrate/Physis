// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;
use binrw::{binread, BinReaderExt, BinResult};
use half::f16;

pub(crate) fn read_bool_from<T: std::convert::From<u8> + std::cmp::PartialEq>(x: T) -> bool {
    x == T::from(1u8)
}

pub(crate) fn write_bool_as<T: std::convert::From<u8>>(x: &bool) -> T {
    if *x {
        T::from(1u8)
    } else {
        T::from(0u8)
    }
}

#[binrw::parser(reader, endian)]
pub(crate) fn strings_parser(base_offset: u64, strings_offset: &Vec<u16>) -> BinResult<Vec<String>> {
    let mut strings: Vec<String> =
        vec![];

    for offset in strings_offset {
        let string_offset = base_offset + *offset as u64;

        let mut string = String::new();

        reader.seek(SeekFrom::Start(string_offset as u64))?;
        let mut next_char = reader.read_le::<u8>().unwrap() as char;
        while next_char != '\0' {
            string.push(next_char);
            next_char = reader.read_le::<u8>().unwrap() as char;
        }

        strings.push(string);
    }

    Ok(strings)
}

fn read_half1(data: [u16; 1]) -> Half1 {
    Half1 {
        value: f16::from_bits(data[0])
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
        y: f16::from_bits(data[0])
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
        g: f16::from_bits(data[0]),
        b: f16::from_bits(data[0])
    }
}

#[binread]
#[derive(Debug, Default, Clone, Copy)]
#[br(map = read_half3)]
pub(crate) struct Half3 {
    pub r: f16,
    pub g: f16,
    pub b: f16
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
}
