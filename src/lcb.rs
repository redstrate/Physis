// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(magic = b"LCB1")]
pub struct Lcb {
    /// Including this header, in bytes.
    file_size: u32,
    /// Number of Lcc's
    #[br(temp)]
    #[bw(calc = lccs.len() as u32)]
    lcc_count: u32,
    #[br(count = lcc_count)]
    pub lccs: Vec<Lcc>,
}

#[binrw]
#[derive(Debug)]
#[brw(magic = b"LCC1")]
pub struct Lcc {
    /// In bytes, including the magic.
    header_size: u32,
    /// Seems to always be 0?
    id: u32,
    pub unk1: u32,

    #[br(temp)]
    #[bw(calc = entries.len() as u32 )]
    num_entries: u32,
    #[br(count = num_entries)]
    pub entries: Vec<LccEntry>,
}

#[binrw]
#[derive(Debug)]
pub struct LccEntry {
    /// Points to a GameObject in this territory.
    pub instance_id: u32,
    // TODO: figure out what this is
    pub unk1: u32,
    pub min: [f32; 3],
    pub max: [f32; 3],
}

impl ReadableFile for Lcb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Lcb::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for Lcb {
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())
                .ok()?;
        }

        Some(buffer)
    }
}
#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Lcb>();
    }
}
