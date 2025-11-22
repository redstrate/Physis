// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct SkpHeader {
    magic: i32, // TODO: what magic?

    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    pub version: String,
}

#[derive(Debug)]
pub struct Skp {}

impl Skp {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        SkpHeader::read(&mut cursor).ok()?;

        Some(Skp {})
    }
}
