// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::binread;
use binrw::BinRead;

#[binread]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum ShaderStage {
    #[br(magic = 0u8)]
    Vertex,
    #[br(magic = 1u8)]
    Pixel,
}

#[binread]
#[derive(Debug)]
#[brw(little)]
#[allow(dead_code)]
struct SchdHeader {
    magic: i32, // TODO: what magic?

    #[br(count = 3)]
    #[bw(pad_size_to = 3)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    version: String,

    stage: ShaderStage,

    dxc_magic: u32, // TODO: WHAT MAGIC??

    file_length: i32,
    shader_offset: u32,
    parameter_offset: u32,
}

#[derive(Debug)]
pub struct Schd {}

impl Schd {
    /// Reads an existing ULD file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        SchdHeader::read(&mut cursor).ok()?;

        Some(Schd {})
    }
}
