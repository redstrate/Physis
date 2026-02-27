// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::string_heap::StringHeap;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

/// Environment binary file, usually with the `.envb` file extension.
#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[brw(magic = b"ENVB")]
pub struct Envb {
    /// Size of the file, including this header.
    file_size: u32,
    chunk_count: u32,

    #[br(count = chunk_count)]
    pub envs: Vec<Envs>,
}

#[binrw]
#[brw(magic = b"ENVS")]
#[derive(Debug)]
pub struct Envs {
    /// Size of this header, in bytes.
    size: u32,
    unk1: u32,
    unk2: u32,
    children_count: u32,
    unk3: u32,
    #[brw(pad_after = 8)] // empty?
    unk4: u32,
}

#[binrw]
#[derive(Debug)]
#[brw(magic = b"007V")]
struct Entry007V {
    f0x24: f32,
    f0x28: f32,
    f0x2c: f32,
    f0x30: f32,
    f0x34: f32,
    f0x38: f32,
    f0x3c: f32,
    f0x40: f32,
    f0x44: f32,
    f0x48: f32,
    f0x4c: f32,
    f0x50: u8,
    f0x51: u8,
    f0x52: u8,
    f0x53: u8,
    f0x54: f32,
}

impl ReadableFile for Envb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);
        let string_heap = StringHeap::from(cursor.position());

        Envb::read_options(&mut cursor, endianness, (&string_heap,)).ok()
    }
}

impl WritableFile for Envb {
    fn write_to_buffer(&self, platform: Platform) -> Option<crate::ByteBuffer> {
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
    use std::{fs::read, path::PathBuf};

    use super::*;

    #[test]
    fn read_empty_envb() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("lenv_s1h1_outdoor.envb");

        let envb = Envb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(envb.envs.len(), 1);

        let envs = &envb.envs[0];
        assert_eq!(envs.children_count, 0);
    }

    #[test]
    fn write_empty_envb() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("lenv_s1h1_outdoor.envb");

        let envb_bytes = read(d).unwrap();
        let env = Envb::from_existing(Platform::Win32, &envb_bytes).unwrap();

        assert_eq!(env.write_to_buffer(Platform::Win32).unwrap(), envb_bytes);
    }
}
