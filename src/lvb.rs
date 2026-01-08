// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(dead_code)]

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::scn::{ScnSection, write_scns};
use crate::string_heap::StringHeap;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

/// Level variable binary file, usually with the `.lvb` file extension.
///
/// Contains general information about the level, such as which layer groups it has.
#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[brw(magic = b"LVB1")]
pub struct Lvb {
    /// Including this header
    file_size: u32,

    /// Number of Scn's
    #[br(temp)]
    #[bw(calc = sections.len() as u32)]
    section_count: u32,

    /// The sections of this file.
    #[br(count = section_count)]
    #[br(args{ inner: (string_heap,) })]
    #[bw(write_with = write_scns, args(string_heap,))]
    pub sections: Vec<ScnSection>,
}

impl ReadableFile for Lvb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let string_heap = StringHeap::from(cursor.position());

        Lvb::read_options(&mut cursor, platform.endianness(), (&string_heap,)).ok()
    }
}

impl WritableFile for Lvb {
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut string_heap = StringHeap::from(0);

            // TODO: need dual pass

            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), (&mut string_heap,))
                .ok()?;

            string_heap
                .write_options(&mut cursor, platform.endianness(), ())
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
        pass_random_invalid::<Lvb>();
    }
}
