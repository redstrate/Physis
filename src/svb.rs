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

/// Sky visibility binary file, usually with the `.svb` file extension.
#[binrw]
#[derive(Debug)]
#[brw(magic = b"SVB1")]
pub struct Svb {
    /// Including this header, in bytes.
    file_size: u32,
    /// Number of Svc's
    #[br(temp)]
    #[bw(calc = svcs.len() as u32)]
    svc_count: u32,
    #[br(count = svc_count)]
    pub svcs: Vec<Svc>,
}

#[binrw]
#[derive(Debug)]
#[brw(magic = b"SVC1")]
pub struct Svc {
    /// In bytes, including the magic.
    header_size: u32,
    /// Seems to always be 0?
    id: u32,
    pub unk1: u32,

    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    pub num_entries: u32,
    #[br(count = num_entries)]
    pub entries: Vec<SvcEntry>,
}

#[binrw]
#[derive(Debug)]
pub struct SvcEntry {
    /// Points to a GameObject in this territory.
    pub instance_id: u32,
    // TODO: figure out what this is
    pub unk2: u32,
    pub unk3: f32,
}

impl ReadableFile for Svb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Svb::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for Svb {
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
        pass_random_invalid::<Svb>();
    }
}
