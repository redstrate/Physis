// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::Write;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::envs::Envs;
use crate::envs::write_envs;
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
    envs_count: u32,

    #[br(count = envs_count, args { inner: (string_heap,) })]
    #[bw(write_with = write_envs, args(&mut string_heap,))]
    pub envs: Vec<Envs>,
}

impl ReadableFile for Envb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);
        let string_heap = StringHeap::from(cursor.position() as i64);

        Ok(Envb::read_options(
            &mut cursor,
            endianness,
            (&string_heap,),
        )?)
    }
}

impl WritableFile for Envb {
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
    use std::{fs::read, path::PathBuf};

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Envb>();
    }

    #[test]
    fn read_empty_envb() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("lenv_s1h1_outdoor.envb");

        let envb = Envb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(envb.envs.len(), 1);

        let envs = &envb.envs[0];
        assert_eq!(envs.sections.len(), 0);
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
