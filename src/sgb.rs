// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::common::Platform;
use crate::layer::ScnSection;
use crate::string_heap::StringHeap;
use crate::{ByteSpan, ReadableFile};
use binrw::BinRead;
use binrw::binrw;

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
        let string_heap = StringHeap::from(cursor.position());

        Sgb::read_options(&mut cursor, endianness, (&string_heap,)).ok()
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
