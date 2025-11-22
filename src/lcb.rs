// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(little)]
#[brw(magic = b"LCB1")]
pub struct Lcb {
    /// Including this header
    pub file_size: u32,
    /// Number of Lcc's
    #[br(temp)]
    #[bw(calc = lccs.len() as u32)]
    lcc_count: u32,
    #[br(count = lcc_count)]
    pub lccs: Vec<Lcc>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
#[brw(magic = b"LCC1")]
pub struct Lcc {
    #[br(temp)]
    #[bw(calc = entries.len() as u32 + 1)]
    pub num_entries: u32,
    #[brw(pad_before = 4)] // empty?
    pub unk1: u32,
    pub unk2: u32,
    // TODO: figure out what this is
    // TODO: why is it -1?
    #[br(count = num_entries - 1)]
    pub entries: Vec<LccEntry>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct LccEntry {
    // TODO: figure out what this is
    #[br(count = 32)]
    pub unk1: Vec<u8>,
}

impl Lcb {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Lcb::read(&mut cursor).ok()
    }

    /// Writes data back to a buffer.
    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_le(&mut cursor).ok()?;
        }

        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Lcb::from_existing(&read(d).unwrap());
    }
}
