// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use binrw::{BinRead, binrw};
use crate::ByteSpan;

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

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct ColorSetInfo {
    #[br(count = 256)]
    data: Vec<u16>,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct ColorSetDyeInfo {
    #[br(count = 16)]
    data: Vec<u16>,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ShaderKey {
    pub category: u32,
    pub value: u32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct Constant {
    constant_id: u32,
    value_offset: u16,
    value_size: u16,
}

// from https://github.com/NotAdam/Lumina/blob/master/src/Lumina/Data/Parsing/MtrlStructs.cs
#[binrw]
#[derive(Debug)]
enum TextureUsage
{
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
    SamplerWhitecapMap
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct Sampler {
    texture_usage: TextureUsage,
    flags: u32, // TODO: unknown
    #[br(pad_after = 3)]
    texture_index: u8,
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
    color_set_info: Option<ColorSetInfo>,

    #[br(if(file_header.data_set_size > 512))]
    color_set_due_info: Option<ColorSetDyeInfo>,

    header: MaterialHeader,

    #[br(count = header.shader_key_count)]
    shader_keys: Vec<ShaderKey>,
    #[br(count = header.constant_count)]
    constants: Vec<Constant>,
    #[br(count = header.sampler_count)]
    samplers: Vec<Sampler>,
    #[br(count = header.shader_value_list_size / 4)]
    shader_values: Vec<f32>,
}

#[derive(Debug)]
pub struct Material {
    pub shader_package_name: String,
    pub texture_paths: Vec<String>,
    pub shader_keys: Vec<ShaderKey>
}

impl Material {
    pub fn from_existing(buffer: ByteSpan) -> Option<Material> {
        let mut cursor = Cursor::new(buffer);
        let mat_data = MaterialData::read(&mut cursor).ok()?;

        println!("{:#?}", mat_data);

        let mut texture_paths = vec![];

        let mut offset = 0;
        for _ in 0..mat_data.file_header.texture_count {
            let mut string = String::new();

            let mut next_char = mat_data.strings[offset as usize] as char;
            while next_char != '\0' {
                string.push(next_char);
                offset += 1;
                next_char = mat_data.strings[offset as usize] as char;
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
            next_char = mat_data.strings[offset as usize] as char;
        }

        Some(Material {
            shader_package_name,
            texture_paths,
            shader_keys: mat_data.shader_keys
        })
    }
}
