// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::binrw;
use binrw::BinRead;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct SgbHeader {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    pub identifier: String,

    file_size: i32,
    total_chunk_count: i32
}

#[derive(Debug)]
pub struct Sgb {

}

impl Sgb {
    /// Reads an existing SGB file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let header = SgbHeader::read(&mut cursor).ok()?;

        Some(Sgb{})
    }
}
