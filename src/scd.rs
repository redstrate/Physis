// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
struct ScdHeader {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    pub file_type: String,

    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    pub sub_type: String,

    version: u32,
    endian_type: u32,
    alignment_bits: u8,
    offset: u16,
    datetime: u64,

    #[br(pad_before = 4)]
    sound_count: u16,
    track_count: u16,
    audio_count: u16,
    number: u16,

    track_offset: u32,
    audio_offset: u32,
    layout_offset: u32,
    routing_offset: u32,
    attribute_offset: u32,

    #[br(pad_after = 2)]
    end_of_file_padding_size: u16,
}

#[derive(Debug)]
pub struct Scd {}

impl ReadableFile for Scd {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        ScdHeader::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        Some(Scd {})
    }
}
