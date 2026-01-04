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
    #[bw(calc = self.calculate_file_size())]
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
    #[bw(calc = Self::HEADER_SIZE)]
    header_size: u32,
    /// Seems to always be 0?
    id: u32,
    /// Always seems to be 12?
    unk1: u32,

    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    pub num_entries: u32,
    #[br(count = num_entries)]
    pub entries: Vec<SvcEntry>,
}

impl Svc {
    pub const HEADER_SIZE: u32 = 20;
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

impl Svb {
    /// Creates an empty SVB.
    pub fn new() -> Self {
        Self {
            svcs: vec![Svc {
                id: 0,
                unk1: 12,
                entries: Vec::new(),
            }],
        }
    }

    fn calculate_file_size(&self) -> u32 {
        // TODO: take entries into account
        12 // LCB1 header
        + Svc::HEADER_SIZE // LCC1 header
    }
}

impl Default for Svb {
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
        pass_random_invalid::<Svb>();
    }

    #[test]
    fn test_write_empty() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty.svb");

        let empty_svb = &read(d).unwrap();
        let svb = Svb::new();

        assert_eq!(*empty_svb, svb.write_to_buffer(Platform::Win32).unwrap());
    }
}
