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

/// Level collision binary file, usually with the `.lcb` file extension.
#[binrw]
#[derive(Debug)]
#[brw(magic = b"LCB1")]
pub struct Lcb {
    /// Including this header, in bytes.
    #[bw(calc = self.calculate_file_size())]
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
    #[bw(calc = Self::HEADER_SIZE)]
    header_size: u32,
    /// Seems to always be 0?
    id: u32,
    /// Always seems to be 12?
    unk1: u32,

    #[br(temp)]
    #[bw(calc = entries.len() as u32 )]
    num_entries: u32,
    #[br(count = num_entries)]
    pub entries: Vec<LccEntry>,
}

impl Lcc {
    pub const HEADER_SIZE: u32 = 20;
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

impl Lcb {
    /// Creates an empty LCB.
    pub fn new() -> Self {
        Self {
            lccs: vec![Lcc {
                id: 0,
                unk1: 12,
                entries: Vec::new(),
            }],
        }
    }

    fn calculate_file_size(&self) -> u32 {
        // TODO: take entries into account
        12 // LCB1 header
        + Lcc::HEADER_SIZE // LCC1 header
    }
}

impl Default for Lcb {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::read, path::PathBuf};

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Lcb>();
    }

    #[test]
    fn test_write_empty() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty.lcb");

        let empty_lcb = &read(d).unwrap();
        let lcb = Lcb::new();

        assert_eq!(*empty_lcb, lcb.write_to_buffer(Platform::Win32).unwrap());
    }
}
