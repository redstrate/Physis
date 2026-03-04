// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused)]

use std::collections::HashMap;
use std::io::{Cursor, SeekFrom};

use crate::common::Platform;
use crate::common_file_operations::Half3;
use crate::{ByteSpan, ReadableFile};
use binrw::{BinRead, BinResult, Endian};
use binrw::{BinReaderExt, binrw};
use half::f16;

/// Maximum number of elements in one row
// NOTE: this is probably wrong for other versions of the game?
const MAX_ELEMENTS: usize = 254;

#[binrw]
#[brw(import(num_colors: usize, num_scalars: usize))]
#[derive(Debug)]
#[allow(dead_code)]
struct StainingTemplateEntry {
    #[br(map = calc_byte_counts, count = num_colors + num_scalars)]
    #[br(dbg)]
    byte_counts: Vec<u16>,

    #[br(parse_with = read_color_array, args(&byte_counts, num_colors))]
    #[bw(ignore)]
    colors: Vec<Half3>,
    #[br(parse_with = read_scalar_array, args(&byte_counts, num_colors, num_scalars))]
    #[bw(ignore)]
    scalars: Vec<u16>,
}

/// Legacy dye information.
#[derive(Debug)]
pub struct LegacyDye {
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub emissive: [f32; 3],
    pub shininess: f32,
    pub specular_mask: f32,
}

impl From<StainingTemplateEntry> for LegacyDye {
    fn from(value: StainingTemplateEntry) -> Self {
        Self {
            diffuse: value.colors[0].into(),
            specular: value.colors[1].into(),
            emissive: value.colors[2].into(),
            shininess: f16::from_bits(value.scalars[0]).to_f32_const(),
            specular_mask: f16::from_bits(value.scalars[1]).to_f32_const(),
        }
    }
}

/// Dawntrail-era dye information.
#[derive(Debug)]
pub struct Dye {
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub emissive: [f32; 3],
    scalar3: f32, // TODO: what is this?
    pub metalness: f32,
    pub roughness: f32,
    pub sheen_rate: f32,
    pub sheen_tint_rate: f32,
    pub sheen_aperture: f32,
    pub anisotropy: f32,
    pub raw_sphere_map_index: u16,
    pub sphere_map_mask: f32,
}

impl From<StainingTemplateEntry> for Dye {
    fn from(value: StainingTemplateEntry) -> Self {
        Self {
            diffuse: value.colors[0].into(),
            specular: value.colors[1].into(),
            emissive: value.colors[2].into(),
            scalar3: f16::from_bits(value.scalars[9]).to_f32_const(),
            metalness: f16::from_bits(value.scalars[1]).to_f32_const(),
            roughness: f16::from_bits(value.scalars[2]).to_f32_const(),
            sheen_rate: f16::from_bits(value.scalars[3]).to_f32_const(),
            sheen_tint_rate: f16::from_bits(value.scalars[4]).to_f32_const(),
            sheen_aperture: f16::from_bits(value.scalars[5]).to_f32_const(),
            anisotropy: f16::from_bits(value.scalars[6]).to_f32_const(),
            raw_sphere_map_index: value.scalars[7],
            sphere_map_mask: f16::from_bits(value.scalars[8]).to_f32_const(),
        }
    }
}

fn calc_byte_counts(ends: Vec<u16>) -> Vec<u16> {
    let mut last_end = 0;
    let mut byte_counts = Vec::new();
    for end in ends {
        let next_end = end * 2;
        byte_counts.push(next_end.wrapping_sub(last_end));
        last_end = next_end;
    }

    byte_counts
}

#[binrw::parser(reader, endian)]
fn read_color_array(byte_counts: &[u16], num_colors: usize) -> BinResult<Vec<Half3>> {
    let mut entries = Vec::with_capacity(num_colors);

    for count in byte_counts.iter().take(num_colors) {
        let mut array: Vec<Half3> = read_array(reader, endian, *count);
        entries.append(&mut array); // TODO: wrong
    }

    Ok(entries)
}

#[binrw::parser(reader, endian)]
fn read_scalar_array(
    byte_counts: &[u16],
    num_colors: usize,
    num_scalars: usize,
) -> BinResult<Vec<u16>> {
    let mut entries = Vec::with_capacity(num_scalars);

    for count in byte_counts.iter().skip(num_colors).take(num_scalars) {
        let mut array: Vec<u16> = read_array(reader, endian, *count);
        entries.append(&mut array); // TODO: wrong
    }

    Ok(entries)
}

fn read_array<
    T: binrw::BinRead<Args<'static> = ()> + Default + Clone + Copy,
    R: std::io::Read + BinReaderExt,
>(
    cursor: &mut R,
    endian: Endian,
    size: u16,
) -> Vec<T> {
    let array_size = size as usize / std::mem::size_of::<T>();
    if array_size == 0 {
        vec![T::default(); MAX_ELEMENTS]
    } else if array_size == 1 {
        let element = cursor.read_type::<T>(endian).unwrap();
        vec![element; MAX_ELEMENTS]
    } else if array_size < MAX_ELEMENTS {
        let real_count = (size as usize - MAX_ELEMENTS) / std::mem::size_of::<T>();
        let mut values = vec![];
        let mut indices = vec![];
        values.push(T::default());
        for _ in 0..real_count {
            values.push(cursor.read_type::<T>(endian).unwrap());
        }

        let eof_marker = cursor.read_type::<u8>(endian).unwrap();
        assert_eq!(eof_marker, 0xFF); // TDOO: restore

        for _ in 0..MAX_ELEMENTS - 1 {
            indices.push(cursor.read_type::<u8>(endian).unwrap());
        }

        let mut vec = vec![];
        for index in indices {
            if (index as usize) < values.len() {
                vec.push(values[index as usize]);
            } else {
                vec.push(T::default());
            }
        }

        vec
    } else if array_size == MAX_ELEMENTS {
        let mut vec = vec![];
        for _ in 0..size {
            vec.push(cursor.read_type::<T>(endian).unwrap());
        }
        vec
    } else {
        panic!("Too many elements");
    }
}

/// Staining template material file, usually with the `.stm` file extension.
///
/// Contains dye information.
#[binrw]
#[derive(Debug)]
#[brw(magic = 0x534Du16)]
pub struct Stm {
    version: u16,
    entry_count: u16,
    unk1: u16,

    // NOTE: older versions i think is u16?
    #[br(count = entry_count)]
    #[br(dbg)]
    keys: Vec<u32>,

    #[br(count = entry_count)]
    #[br(dbg)]
    offsets: Vec<u32>,

    #[br(calc = 3)]
    #[bw(ignore)]
    num_colors: usize,
    #[br(calc = if version == 0x101 { 2 } else { 9 } )]
    #[bw(ignore)]
    num_scalars: usize,

    #[br(if(version != 0x101), parse_with = read_entries, args(&keys, &offsets, num_colors, num_scalars))]
    #[bw(ignore)] // TODO: support writing
    pub dyes: HashMap<u32, Dye>,

    #[br(if(version == 0x101), parse_with = read_entries, args(&keys, &offsets, num_colors, num_scalars))]
    #[bw(ignore)] // TODO: support writing
    pub legacy_dyes: HashMap<u32, LegacyDye>,
}

#[binrw::parser(reader, endian)]
fn read_entries<T: From<StainingTemplateEntry>>(
    keys: &[u32],
    offsets: &[u32],
    num_colors: usize,
    num_scalars: usize,
) -> BinResult<HashMap<u32, T>> {
    let mut entries = HashMap::with_capacity(keys.len());

    let start_position = reader.stream_position().unwrap();

    for (key, offset) in keys.iter().zip(offsets) {
        reader.seek(SeekFrom::Start(start_position + *offset as u64 * 2))?;
        let entry: StainingTemplateEntry =
            reader.read_type_args(endian, (num_colors, num_scalars))?;
        entries.insert(*key, T::from(entry));
    }

    Ok(entries)
}

impl ReadableFile for Stm {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Stm::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Stm>();
    }
}
