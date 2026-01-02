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
struct SgbHeader {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    pub identifier: String,

    file_size: i32,
    total_chunk_count: i32,
}

/// Shared group binary file, usually with the `.sgb` file extension.
///
/// This is basically a "prefab".
#[derive(Debug)]
pub struct Sgb {}

impl ReadableFile for Sgb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        SgbHeader::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        Some(Sgb {})
    }
}
