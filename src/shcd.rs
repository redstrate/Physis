// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::binread;

#[binread]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum ShaderStage {
    #[br(magic = 0u8)]
    Vertex,
    #[br(magic = 1u8)]
    Pixel,
}

/// Shader file, usually with the `.shcd` file extension.
///
/// Used instead of a shader package for standalone shaders.
#[binread]
#[derive(Debug)]
#[brw(magic = b"ShCd")]
#[allow(dead_code)]
pub struct SHCD {
    version: i32,

    // "DX9\0" or "DX11"
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    format: String,

    file_length: u32,
    shader_offset: u32,
    parameter_offset: u32,
    #[brw(pad_size_to = 4)]
    stage: ShaderStage,
    shader_data_length: u32,

    // TODO: there's other interesting data in here
    // TODO: read parameters
    /// DXBC bytecode.
    #[br(seek_before = SeekFrom::Start(shader_offset as u64))]
    #[br(count = shader_data_length)]
    #[br(restore_position)]
    pub bytecode: Vec<u8>,
}

impl ReadableFile for SHCD {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        SHCD::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}
