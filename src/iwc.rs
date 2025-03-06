// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct IwcHeader {
    count: u16,
    part_mask: u16,
}

#[derive(Debug)]
pub struct Iwc {}

impl Iwc {
    /// Reads an existing ULD file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        IwcHeader::read(&mut cursor).ok()?;

        Some(Iwc {})
    }
}
