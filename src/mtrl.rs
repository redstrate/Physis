// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::unnecessary_fallible_conversions)] // This wrongly trips on binrw code

use std::io::Cursor;

use crate::common_file_operations::{Half1, Half2, Half3};
use crate::ByteSpan;
use binrw::{binread, binrw, BinRead};

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
    #[br(pad_after = 4)]
    sampler_count: u16,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct ColorSet {
    name_offset: u16,
    #[br(pad_after = 1)]
    index: u8,
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub struct ColorTableRow {
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
#[br(import {set_count: usize})]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ColorTable {
    #[br(count = set_count)]
    pub rows: Vec<ColorTableRow>,
}

#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ColorDyeTableRow {
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
pub struct ColorDyeTable {
    #[br(count = 16)]
    pub rows: Vec<ColorDyeTableRow>,
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

// from https://github.com/NotAdam/Lumina/blob/master/src/Lumina/Data/Parsing/MtrlStructs.cs
#[binrw]
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum TextureUsage {
    #[brw(magic = 0x88408C04u32)]
    Sampler,
    #[brw(magic = 0x213CB439u32)]
    Sampler0,
    #[brw(magic = 0x563B84AFu32)]
    Sampler1,
    #[brw(magic = 0xFEA0F3D2u32)]
    SamplerCatchlight,
    #[brw(magic = 0x1E6FEF9Cu32)]
    SamplerColorMap0,
    #[brw(magic = 0x6968DF0Au32)]
    SamplerColorMap1,
    #[brw(magic = 0x115306BEu32)]
    SamplerDiffuse,
    #[brw(magic = 0xF8D7957Au32)]
    SamplerEnvMap,
    #[brw(magic = 0x8A4E82B6u32)]
    SamplerMask,
    #[brw(magic = 0x0C5EC1F1u32)]
    SamplerNormal,
    #[brw(magic = 0xAAB4D9E9u32)]
    SamplerNormalMap0,
    #[brw(magic = 0xDDB3E97Fu32)]
    SamplerNormalMap1,
    #[brw(magic = 0x87F6474Du32)]
    SamplerReflection,
    #[brw(magic = 0x2B99E025u32)]
    SamplerSpecular,
    #[brw(magic = 0x1BBC2F12u32)]
    SamplerSpecularMap0,
    #[brw(magic = 0x6CBB1F84u32)]
    SamplerSpecularMap1,
    #[brw(magic = 0xE6321AFCu32)]
    SamplerWaveMap,
    #[brw(magic = 0x574E22D6u32)]
    SamplerWaveletMap0,
    #[brw(magic = 0x20491240u32)]
    SamplerWaveletMap1,
    #[brw(magic = 0x95E1F64Du32)]
    SamplerWhitecapMap,

    #[brw(magic = 0x565f8fd8u32)]
    UnknownDawntrail1,

    #[brw(magic = 0xe5338c17u32)]
    UnknownDawntrail2,
}

#[binrw]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(dead_code)]
pub struct Sampler {
    texture_usage: TextureUsage,
    flags: u32, // TODO: unknown
    texture_index: u8,
    unknown1: u8,
    unknown2: u8,
    unknown3: u8,
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
    #[br(pad_after = file_header.additional_data_size)]
    strings: Vec<u8>,

    #[br(if(file_header.data_set_size > 0))]
    // Dawntrail doubled the amount of color sets.
    // The MTRL version is the same (why square enix?) so we check the data set size instead
    #[br(args { set_count: if file_header.data_set_size < 2048 { 16 } else { 32 } })]
    #[bw(ignore)]
    color_table: Option<ColorTable>,

    #[br(if(file_header.data_set_size >
        if file_header.data_set_size < 2048 { 512 } else { 2048 }
    ))]
    #[bw(ignore)]
    color_dye_table: Option<ColorDyeTable>,

    header: MaterialHeader,

    #[br(count = header.shader_key_count)]
    shader_keys: Vec<ShaderKey>,
    #[br(count = header.constant_count)]
    constants: Vec<ConstantStruct>,
    #[br(count = header.sampler_count)]
    samplers: Vec<Sampler>,
    #[br(count = header.shader_value_list_size / 4)]
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

        let mut constants = Vec::new();
        for constant in mat_data.constants {
            let mut values: [f32; 4] = [0.0; 4];

            // TODO: use mem::size_of
            let num_floats = constant.value_size / 4;
            for i in 0..num_floats as usize {
                values[i] = mat_data.shader_values[(constant.value_offset as usize / 4) + i];
            }

            constants.push(Constant {
                id: constant.constant_id,
                num_values: num_floats as u32,
                values,
            });
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
