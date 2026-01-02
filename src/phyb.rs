// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::binread;

#[binread]
#[derive(Debug)]
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

impl ReadableFile for Phyb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        PhybHeader::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        Some(Phyb {})
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Phyb>();
    }
}
