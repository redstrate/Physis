// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Read, Seek, SeekFrom};

use binrw::{BinRead, binread};
use crate::ByteSpan;

#[binread]
#[br(little)]
#[derive(Debug)]
#[allow(unused)]
struct ParameterHeader {
    id: i32,
    name_offset: i32,
    #[br(pad_after = 8)]
    name_length: i32
}

#[binread]
#[br(little)]
#[derive(Debug)]
#[allow(unused)]
struct ShaderParameterReference {
    #[br(pad_after = 12)]
    id: i32
}

#[binread]
#[br(little)]
#[derive(Debug)]
#[allow(unused)]
struct ShaderHeader {
    data_offset: i32,
    data_length: i32,

    num_scalar: i16,
    #[br(pad_after = 4)]
    num_resource: i16,

    #[br(count = num_scalar)]
    scalar_parameters: Vec<ShaderParameterReference>,
    #[br(count = num_resource)]
    resource_parameters: Vec<ShaderParameterReference>,
}

#[binread]
#[br(little)]
#[br(magic = b"ShPk")]
#[derive(Debug)]
#[allow(dead_code)]
struct SHPKHeader {
    #[br(pad_before = 4)] // what are these bytes? 01 0B
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    format: String,

    file_length: i32,
    shader_data_offset: i32,
    parameter_list_offset: i32,
    vertex_shader_count: i32,
    pixel_shader_count: i32,

    #[br(pad_before = 4)]
    c1: i32,

    // 8 bytes...
    scalar_parameter_count: i32,
    resource_parameter_count: i32,

    #[br(pad_before = 24)]
    #[br(count = vertex_shader_count)]
    vertex_shader_headers: Vec<ShaderHeader>,
    #[br(count = pixel_shader_count)]
    pixel_shader_headers: Vec<ShaderHeader>
}

pub struct Shader {
    /// The HLSL bytecode of this shader. The DX level used varies.
    pub bytecode: Vec<u8>
}

pub struct ShaderPackage {
    /// The vertex shaders in this package
    pub vertex_shaders: Vec<Shader>,
    /// The pixel (fragment) shaders in this package
    pub pixel_shaders: Vec<Shader>
}

impl ShaderPackage {
    /// Reads an existing SHPK file
    pub fn from_existing(buffer: ByteSpan) -> Option<ShaderPackage> {
        let mut cursor = Cursor::new(buffer);
        let shpk_header = SHPKHeader::read(&mut cursor).unwrap();

        // shader parameters
        cursor.seek(SeekFrom::Current((shpk_header.c1 as u64 * 0x08) as i64)).ok()?;

        for _ in 0..shpk_header.scalar_parameter_count {
            let _ = ParameterHeader::read_le(&mut cursor).ok()?;
        }

        for _ in 0..shpk_header.resource_parameter_count {
            let _ = ParameterHeader::read_le(&mut cursor).ok()?;
        }

        // shader bytecode
        let mut vertex_shaders: Vec<Shader> = Vec::new();
        for header in shpk_header.vertex_shader_headers {
            cursor.seek(SeekFrom::Start((shpk_header.shader_data_offset + header.data_offset + 8) as u64)).ok()?;

            let mut bytecode = vec![0u8; header.data_length as usize];
            cursor.read_exact(bytecode.as_mut_slice()).ok()?;

            vertex_shaders.push(Shader {
                bytecode
            });
        }

        let mut pixel_shaders: Vec<Shader> = Vec::new();
        for header in shpk_header.pixel_shader_headers {
            cursor.seek(SeekFrom::Start((shpk_header.shader_data_offset + header.data_offset) as u64)).ok()?;

            let mut bytecode = vec![0u8; header.data_length as usize];
            cursor.read_exact(bytecode.as_mut_slice()).ok()?;

            pixel_shaders.push(Shader {
                bytecode
            });
        }

        Some(ShaderPackage {
            vertex_shaders,
            pixel_shaders
        })
    }
}
