// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use binrw::{BinRead, BinWrite, binrw};

use crate::{ByteBuffer, ByteSpan, ReadableFile, WritableFile, common::Platform, pcb::AABB};

/// Collision streaming file, always called `list.pcb`. Despite the name, it's a completely different format to normal PCBs.
#[binrw]
#[derive(Debug, Default)]
pub struct PcbList {
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    entry_count: u32,
    pub bounds: AABB,
    #[brw(pad_before = 4)] // empty padding
    #[br(count = entry_count)]
    pub entries: Vec<PcbListEntry>,
}

/// Represents a PCB file in the same directory, with the filename `tr{mesh_id}.pcb`.
#[binrw]
#[derive(Debug)]
pub struct PcbListEntry {
    pub mesh_id: u32,
    #[brw(pad_before = 4)] // empty padding
    pub bounds: AABB,
}

impl ReadableFile for PcbList {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        PcbList::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for PcbList {
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
    use std::{fs::read, path::PathBuf};

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<PcbList>();
    }

    #[test]
    fn test_write_empty() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty_list.pcb");

        let empty_pcb = &read(d).unwrap();
        let pcb = PcbList::default();

        assert_eq!(*empty_pcb, pcb.write_to_buffer(Platform::Win32).unwrap());
    }
}
