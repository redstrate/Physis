// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::common_file_operations::read_string_until_null;
use crate::common_file_operations::write_string;
use binrw::binrw;
use binrw::{BinRead, BinWrite};

#[binrw]
#[derive(Debug)]
pub struct EidBindPointOld {
    id: i32,
    position: [f32; 3],
    rotation: [f32; 3],
    #[brw(pad_size_to = 12)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    padding_probably: i32,
}

#[binrw]
#[derive(Debug)]
pub struct EidBindPointNew {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    id: i32,
    position: [f32; 3],
    rotation: [f32; 3],
    padding_probably: i32,
}

/// Bind point file, usually with the `.eid` file extension.
#[binrw]
#[derive(Debug)]
#[brw(magic = b"die\0")] // no :-(
pub struct Eid {
    version1: i16,
    version2: i16,
    count: i32,
    unk1: u32,

    #[br(if(version1 != 0x3132), count = count)]
    old_bind_points: Vec<EidBindPointOld>,

    #[br(if(version1 == 0x3132), count = count)]
    new_bind_points: Vec<EidBindPointNew>,
}

impl ReadableFile for Eid {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);

        Self::read_options(&mut cursor, endianness, ()).ok()
    }
}

impl WritableFile for Eid {
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
        pass_random_invalid::<Eid>();
    }
}
