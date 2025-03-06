// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::BinRead;
use binrw::binread;

#[binread]
#[derive(Debug)]
#[brw(little)]
#[allow(dead_code)]
struct PhybHeader {
    version: [u8; 4],

    // TODO: this is definitely wrong
    #[br(if(version[0] > 0))]
    data_type: u32,

    collision_offset: u32,
    simulator_offset: u32,
}

#[derive(Debug)]
pub struct Phyb {}

impl Phyb {
    /// Reads an existing ULD file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        PhybHeader::read(&mut cursor).ok()?;

        Some(Phyb {})
    }
}
