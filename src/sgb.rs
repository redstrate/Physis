// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::common::Platform;
use crate::scn::ScnSection;
use crate::string_heap::StringHeap;
use crate::{ByteBuffer, ByteSpan, ReadableFile, WritableFile};
use binrw::binrw;
use binrw::{BinRead, BinWrite};

/// Shared group binary file, usually with the `.sgb` file extension.
///
/// This is basically a "prefab".
#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[brw(magic = b"SGB1")]
#[derive(Debug)]
pub struct Sgb {
    file_size: i32,
    total_chunk_count: i32,
    #[br(count = total_chunk_count, args { inner: (string_heap,) })]
    #[bw(ignore)] // TODO: support writing
    pub sections: Vec<ScnSection>,
}

impl ReadableFile for Sgb {
    /// Read an existing file.
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);
        let string_heap = StringHeap::from(cursor.position() as i64);

        Sgb::read_options(&mut cursor, endianness, (&string_heap,)).ok()
    }
}

impl WritableFile for Sgb {
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            let mut string_heap = StringHeap::from(cursor.position() as i64);
            self.write_options(&mut cursor, platform.endianness(), (&mut string_heap,))
                .ok()?;

            // TODO: write string heap
        }

        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pass_random_invalid;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Sgb>();
    }
}
