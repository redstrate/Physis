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
#[brw(magic = b"UWB1")]
pub struct Uwb {
    /// Including this header
    pub file_size: u32,
    /// Number of UWC's
    #[br(temp)]
    #[bw(calc = uwcs.len() as u32)]
    uwc_count: u32,
    #[br(count = uwc_count)]
    pub uwcs: Vec<Uwc>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
#[brw(magic = b"UWC1")]
pub struct Uwc {
    /// Including this header
    pub file_size: u32,
    // TODO: figure out what this is
    #[br(count = file_size - 8)]
    pub unk1: Vec<u8>,
}

impl Uwb {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Uwb::read(&mut cursor).ok()
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
        Uwb::from_existing(&read(d).unwrap());
    }
}
