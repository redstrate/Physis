// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct TmbHeader {
    magic: i32, // TODO: figure out what this
    size: i32,
    entry_count: i32,
}

#[derive(Debug)]
pub struct Tmb {}

impl Tmb {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        TmbHeader::read(&mut cursor).ok()?;

        Some(Tmb {})
    }
}
