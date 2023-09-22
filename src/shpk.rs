// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use binrw::{BinRead, binread, BinReaderExt};

use crate::gamedata::MemoryBuffer;

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
    scalar_parameter_count: i32,
    resource_parameter_count: i32
}

pub struct Shader {
    pub bytecode: Vec<u8>
}

pub struct ShaderPackage {
    pub vertex_shaders: Vec<Shader>,
    pub pixel_shaders: Vec<Shader>
}

impl ShaderPackage {
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<ShaderPackage> {
        let mut cursor = Cursor::new(buffer);
        let header = SHPKHeader::read(&mut cursor).ok()?;

        // start of shader data
        cursor.seek(SeekFrom::Start(header.shader_data_offset as u64 + 12)).ok()?;

        let mut vertex_shaders: Vec<Shader> = Vec::new();
        let mut pixel_shaders: Vec<Shader> = Vec::new();

        let mut buffer: Vec<u8> = Vec::new();
        let eof_magic = 1128421444u32;

        while vertex_shaders.len() < header.vertex_shader_count as usize {
            let word = cursor.read_le::<u32>().unwrap();

            if word == eof_magic && !buffer.is_empty() {
                vertex_shaders.push(Shader {
                    bytecode: buffer.clone()
                });
                buffer.clear();
            } else {
                buffer.extend_from_slice(word.to_le_bytes().as_slice());
            }
        }

        while pixel_shaders.len() < header.pixel_shader_count as usize {
            let word = cursor.read_le::<u32>().unwrap();

            if word == eof_magic && !buffer.is_empty() || cursor.position() == header.parameter_list_offset as u64 {
                pixel_shaders.push(Shader {
                    bytecode: buffer.clone()
                });
                buffer.clear();
            } else {
                buffer.extend_from_slice(word.to_le_bytes().as_slice());
            }
        }

        Some(ShaderPackage {
            vertex_shaders,
            pixel_shaders
        })
    }
}
