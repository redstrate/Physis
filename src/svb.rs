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
#[brw(magic = b"SVB1")]
pub struct Svb {
    /// Including this header
    pub file_size: u32,
    /// Number of Svc's
    #[br(temp)]
    #[bw(calc = svcs.len() as u32)]
    svc_count: u32,
    #[br(count = svc_count)]
    pub svcs: Vec<Svc>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
#[brw(magic = b"SVC1")]
pub struct Svc {
    #[br(temp)]
    #[bw(calc = entries.len() as u32 + 1)]
    pub num_entries: u32,
    #[brw(pad_before = 4)] // empty?
    pub unk1: u32,
    pub unk2: u32,
    // TODO: figure out what this is
    // TODO: why is it -1?
    #[br(count = num_entries - 1)]
    pub entries: Vec<SvcEntry>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct SvcEntry {
    // TODO: figure out what this is
    #[br(count = 48)]
    pub unk1: Vec<u8>,
}

impl Svb {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Svb::read(&mut cursor).ok()
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
        Svb::from_existing(&read(d).unwrap());
    }
}
