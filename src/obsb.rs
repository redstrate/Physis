// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::Write;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::envs::EnvsHeader;
use crate::envs::write_envs;
use crate::string_heap::StringHeap;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

/// Object behavior set binary, usually with the `.obsb` file extension.
#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[brw(magic = b"OBSB")]
pub struct Obsb {
    /// Size of the file, including this header.
    file_size: u32,
    envs_count: u32,

    #[br(count = envs_count, args { inner: (string_heap,) })]
    #[bw(write_with = write_envs, args(&mut string_heap,))]
    pub envs: Vec<EnvsHeader>,
}

impl ReadableFile for Obsb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);
        let string_heap = StringHeap::from(cursor.position() as i64);

        Ok(Obsb::read_options(
            &mut cursor,
            endianness,
            (&string_heap,),
        )?)
    }
}

impl WritableFile for Obsb {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut string_heap = StringHeap::from(0);

            // TODO: need dual pass

            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), (&mut string_heap,))?;

            string_heap.write_options(&mut cursor, platform.endianness(), ())?;

            let unk_ending = &[0x0; 8];
            cursor.write_all(unk_ending)?;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Obsb>();
    }
}
