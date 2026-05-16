// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinReaderExt;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::VecArgs;
use binrw::binrw;

/// Ambient set file, usually with the `.amb` file extension.
#[binrw]
#[derive(Debug)]
#[brw(magic = b"AMB\0")]
pub struct Amb {
    /// Should be 1?
    unk1: u16,
    /// Should be 0?
    unk2: u8,
    /// Should be 0? If it isn't probably an old file or something?
    unk3: u8,

    #[br(count = 36)] // full of something interesting i'm sure
    unk4: Vec<u8>,

    inner_counts: [i32; Self::ENTRY_COUNT],

    #[br(parse_with = read_amb_entry, args(&inner_counts,))]
    pub entries: [Vec<AmbEntry>; Self::ENTRY_COUNT],
}

impl Amb {
    pub(crate) const ENTRY_COUNT: usize = 0x20;
}

#[binrw::parser(reader, endian)]
fn read_amb_entry(counts: &[i32]) -> BinResult<[Vec<AmbEntry>; Amb::ENTRY_COUNT]> {
    let mut entries: [Vec<AmbEntry>; Amb::ENTRY_COUNT] = Default::default();

    for (i, count) in counts.iter().enumerate() {
        let entry: Vec<AmbEntry> =
            reader.read_type_args(endian, VecArgs::builder().count(*count as usize).finalize())?;
        entries[i] = entry;
    }

    Ok(entries)
}

/// Entry into an [Amb] file.
#[binrw]
#[derive(Debug)]
pub struct AmbEntry {
    unk1: f32,
    unk2: f32,
    unk3: f32,
    unk4: f32,

    unk5: u64,
    unk6: u64,
    unk7: u64,
    unk8: u64,
    unk9: u64,
    unk10: u64,
    unk11: u64,
    unk12: u64,
    unk13: u64,
    unk14: u64,
    unk15: u64,
    unk16: u64,
}

impl ReadableFile for Amb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);

        Amb::read_options(&mut cursor, endianness, ()).ok()
    }
}

impl WritableFile for Amb {
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
        pass_random_invalid::<Amb>();
    }
}
