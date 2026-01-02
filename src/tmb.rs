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
struct TmbHeader {
    magic: i32, // TODO: figure out what this
    size: i32,
    entry_count: i32,
}

#[derive(Debug)]
pub struct Tmb {}

impl ReadableFile for Tmb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        TmbHeader::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        Some(Tmb {})
    }
}
