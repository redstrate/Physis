// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::binrw;
use binrw::BinRead;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct StmHeader {
    #[br(pad_before = 1)] // TODO: what is this byte?
    entry_count: i32
}

#[derive(Debug)]
pub struct Stm {

}

impl Stm {
    /// Reads an existing ULD file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let header = StmHeader::read(&mut cursor).ok()?;

        Some(Stm{})
    }
}
