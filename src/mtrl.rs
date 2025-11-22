// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::unnecessary_fallible_conversions)] // This wrongly trips on binrw code

use std::io::Cursor;

use crate::ByteSpan;
use crate::common_file_operations::{Half1, Half2, Half3};
use crate::mtrl::ColorDyeTable::{
    DawntrailColorDyeTable, LegacyColorDyeTable, OpaqueColorDyeTable,
};
use crate::mtrl::ColorTable::{DawntrailColorTable, LegacyColorTable, OpaqueColorTable};
use binrw::{BinRead, BinResult, binread, binrw};

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct MaterialFileHeader {
    version: u32,
    file_size: u16,
    data_set_size: u16,
    string_table_size: u16,
    shader_package_name_offset: u16,
    texture_count: u8,
    uv_set_count: u8,
    color_set_count: u8,
    additional_data_size: u8,
}

#[binrw]
#[derive(Debug)]
struct MaterialHeader {
    shader_value_list_size: u16,
    shader_key_count: u16,
    constant_count: u16,
    sampler_count: u16,
    flags: u32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct ColorSet {
    name_offset: u16,
    index: u16,
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub struct DawntrailColorTableRow {
    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub diffuse_color: [f32; 3],

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown1: f32,

    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub specular_color: [f32; 3],

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown2: f32,

    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub emissive_color: [f32; 3],

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown3: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub sheen_rate: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub sheen_tint: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub sheen_aperture: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown4: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub roughness: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown5: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub metalness: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub anisotropy: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown6: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub sphere_mask: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown7: f32,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub unknown8: f32,

    pub shader_index: u16,

    pub tile_set: u16,

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub tile_alpha: f32,

    pub sphere_index: u16,

    #[br(map = |x: Half2| { [x.x.to_f32(), x.y.to_f32()] })]
    pub material_repeat: [f32; 2],

    #[br(map = |x: Half2| { [x.x.to_f32(), x.y.to_f32()] })]
    pub material_skew: [f32; 2],
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub struct LegacyColorTableRow {
    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub diffuse_color: [f32; 3],

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub specular_strength: f32,

    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub specular_color: [f32; 3],

    #[br(map = |x: Half1| { x.value.to_f32() })]
    pub gloss_strength: f32,

    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub emissive_color: [f32; 3],

    pub tile_set: u16,

    #[br(map = |x: Half2| { [x.x.to_f32(), x.y.to_f32()] })]
    pub material_repeat: [f32; 2],

    #[br(map = |x: Half2| { [x.x.to_f32(), x.y.to_f32()] })]
    pub material_skew: [f32; 2],
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct LegacyColorTableData {
    #[br(count = 16)]
    pub rows: Vec<LegacyColorTableRow>,
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct DawntrailColorTableData {
    #[br(count = 32)]
    pub rows: Vec<DawntrailColorTableRow>,
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct OpaqueColorTableData {
    // TODO: Support
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub enum ColorTable {
    LegacyColorTable(LegacyColorTableData),
    DawntrailColorTable(DawntrailColorTableData),
    OpaqueColorTable(OpaqueColorTableData),
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct LegacyColorDyeTableRow {
    #[br(temp)]
    data: u16,

    #[br(calc = data >> 5)]
    pub template: u16,

    #[br(calc = (data & 0x01) != 0)]
    pub diffuse: bool,

    #[br(calc = (data & 0x02) != 0)]
    pub specular: bool,

    #[br(calc = (data & 0x04) != 0)]
    pub emissive: bool,

    #[br(calc = (data & 0x08) != 0)]
    pub gloss: bool,

    #[br(calc = (data & 0x10) != 0)]
    pub specular_strength: bool,
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct DawntrailColorDyeTableRow {
    #[br(temp)]
    data: u32,

    #[br(calc = ((data >> 16) & 0x7FF) as u16)]
    pub template: u16,

    #[br(calc = ((data >> 27) & 0x3) as u8)]
    pub channel: u8,

    #[br(calc = (data & 0x0001) != 0)]
    pub diffuse: bool,

    #[br(calc = (data & 0x0002) != 0)]
    pub specular: bool,

    #[br(calc = (data & 0x0004) != 0)]
    pub emissive: bool,

    #[br(calc = (data & 0x0008) != 0)]
    pub scalar3: bool,

    #[br(calc = (data & 0x0010) != 0)]
    pub metalness: bool,

    #[br(calc = (data & 0x0020) != 0)]
    pub roughness: bool,

    #[br(calc = (data & 0x0040) != 0)]
    pub sheen_rate: bool,

    #[br(calc = (data & 0x0080) != 0)]
    pub sheen_tint_rate: bool,

    #[br(calc = (data & 0x0100) != 0)]
    pub sheen_aperture: bool,

    #[br(calc = (data & 0x0200) != 0)]
    pub anisotropy: bool,

    #[br(calc = (data & 0x0400) != 0)]
    pub sphere_map_index: bool,

    #[br(calc = (data & 0x0800) != 0)]
    pub sphere_map_mask: bool,
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct LegacyColorDyeTableData {
    #[br(count = 16)]
    pub rows: Vec<LegacyColorDyeTableRow>,
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct DawntrailColorDyeTableData {
    #[br(count = 32)]
    pub rows: Vec<DawntrailColorDyeTableRow>,
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct OpaqueColorDyeTableData {
    // TODO: implement
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub enum ColorDyeTable {
    LegacyColorDyeTable(LegacyColorDyeTableData),
    DawntrailColorDyeTable(DawntrailColorDyeTableData),
    OpaqueColorDyeTable(OpaqueColorDyeTableData),
}

#[binrw]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub struct ShaderKey {
    pub category: u32,
    pub value: u32,
}

#[binrw]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct ConstantStruct {
    constant_id: u32,
    value_offset: u16,
    value_size: u16,
}

#[derive(Debug, Clone)]
#[repr(C)]
#[allow(dead_code)]
pub struct Constant {
    id: u32,
    num_values: u32,
    values: [f32; 4],
}

#[binrw]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub struct Sampler {
    /// This is a CRC hash, it can be calculated via ShaderPackage::crc
    pub texture_usage: u32,
    flags: u32, // TODO: unknown
    #[brw(pad_after = 3)] // empty bytes/padding
    texture_index: u8,
}

#[binrw::parser(reader, endian)]
fn parse_color_table(table_dimension_logs: u8) -> BinResult<Option<ColorTable>> {
    Ok(Some(match table_dimension_logs {
        0 | 0x42 => LegacyColorTable(LegacyColorTableData::read_options(reader, endian, ())?),
        0x53 => DawntrailColorTable(DawntrailColorTableData::read_options(reader, endian, ())?),
        _ => OpaqueColorTable(OpaqueColorTableData::read_options(reader, endian, ())?),
    }))
}

#[binrw::parser(reader, endian)]
fn parse_color_dye_table(table_dimension_logs: u8) -> BinResult<Option<ColorDyeTable>> {
    Ok(Some(match table_dimension_logs {
        0 => LegacyColorDyeTable(LegacyColorDyeTableData::read_options(reader, endian, ())?),
        0x50...0x5F => DawntrailColorDyeTable(DawntrailColorDyeTableData::read_options(
            reader,
            endian,
            (),
        )?),
        _ => OpaqueColorDyeTable(OpaqueColorDyeTableData::read_options(reader, endian, ())?),
    }))
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
#[br(little)]
struct MaterialData {
    file_header: MaterialFileHeader,

    #[br(count = file_header.texture_count)]
    offsets: Vec<u32>,

    #[br(count = file_header.uv_set_count)]
    uv_color_sets: Vec<ColorSet>,

    #[br(count = file_header.color_set_count)]
    color_sets: Vec<ColorSet>,

    #[br(count = file_header.string_table_size)]
    strings: Vec<u8>,

    #[br(count = file_header.additional_data_size)]
    #[br(pad_size_to = 4)]
    #[br(map = |x: Vec<u8>| if x.len() >= 4 { u32::from_le_bytes(x[0..4].try_into().unwrap()) } else { 0 })]
    table_flags: u32,

    #[br(calc = (table_flags & 0x4) != 0)]
    #[bw(ignore)]
    has_table: bool,

    #[br(calc = (table_flags & 0x8) != 0)]
    #[bw(ignore)]
    has_dye_table: bool,

    #[br(calc = ((table_flags >> 4) & 0xF) as u8)]
    #[bw(ignore)]
    table_width_log: u8,

    #[br(calc = ((table_flags >> 8) & 0xF) as u8)]
    #[bw(ignore)]
    table_height_log: u8,

    #[br(calc = (table_flags >> 4) as u8)]
    #[bw(ignore)]
    table_dimension_logs: u8,

    #[br(calc = !has_table || table_width_log != 0 && table_height_log != 0)]
    #[bw(ignore)]
    is_dawntrail: bool,

    #[br(if(has_table))]
    #[br(parse_with = parse_color_table)]
    #[br(args(table_dimension_logs))]
    #[bw(ignore)]
    color_table: Option<ColorTable>,

    #[br(if(has_dye_table))]
    #[br(parse_with = parse_color_dye_table)]
    #[br(args(table_dimension_logs))]
    #[bw(ignore)]
    color_dye_table: Option<ColorDyeTable>,

    header: MaterialHeader,

    #[br(count = header.shader_key_count)]
    shader_keys: Vec<ShaderKey>,
    #[br(count = header.constant_count)]
    constants: Vec<ConstantStruct>,
    #[br(count = header.sampler_count)]
    samplers: Vec<Sampler>,
    #[br(count = header.shader_value_list_size as usize / std::mem::size_of::<f32>())]
    shader_values: Vec<f32>,
}

#[derive(Debug)]
pub struct Material {
    pub shader_package_name: String,
    pub texture_paths: Vec<String>,
    pub shader_keys: Vec<ShaderKey>,
    pub constants: Vec<Constant>,
    pub samplers: Vec<Sampler>,
    pub color_table: Option<ColorTable>,
    pub color_dye_table: Option<ColorDyeTable>,
}

impl Material {
    pub fn from_existing(buffer: ByteSpan) -> Option<Material> {
        let mut cursor = Cursor::new(buffer);
        let mat_data = MaterialData::read(&mut cursor).ok()?;

        let mut texture_paths = vec![];

        let mut offset = 0;
        for _ in 0..mat_data.file_header.texture_count {
            let mut string = String::new();

            let mut next_char = mat_data.strings[offset] as char;
            while next_char != '\0' {
                string.push(next_char);
                offset += 1;
                next_char = mat_data.strings[offset] as char;
            }

            texture_paths.push(string);

            offset += 1;
        }

        // TODO: move to reusable function
        let mut shader_package_name = String::new();

        offset = mat_data.file_header.shader_package_name_offset as usize;

        let mut next_char = mat_data.strings[offset] as char;
        while next_char != '\0' {
            shader_package_name.push(next_char);
            offset += 1;
            next_char = mat_data.strings[offset] as char;
        }

        // bg/ffxiv/wil_w1/evt/w1eb/material/w1eb_f1_vfog1a.mtrl has a shader value list of 9, which doesn't make sense in this system
        // eventually we need to un-hardcode it from vec4 or whatever
        let mut constants = Vec::new();
        const VALUE_SIZE: u16 = std::mem::size_of::<f32>() as u16;
        if mat_data.header.shader_value_list_size % VALUE_SIZE == 0 {
            for constant in mat_data.constants {
                let mut values: [f32; 4] = [0.0; 4];

                // TODO: use mem::size_of
                let num_floats = constant.value_size / VALUE_SIZE;
                for (i, value) in values.iter_mut().enumerate().take(num_floats as usize) {
                    *value = mat_data.shader_values[(constant.value_offset as usize / 4) + i];
                }

                constants.push(Constant {
                    id: constant.constant_id,
                    num_values: num_floats as u32,
                    values,
                });
            }
        }

        Some(Material {
            shader_package_name,
            texture_paths,
            shader_keys: mat_data.shader_keys,
            constants,
            samplers: mat_data.samplers,
            color_table: mat_data.color_table,
            color_dye_table: mat_data.color_dye_table,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Material::from_existing(&read(d).unwrap());
    }
}
