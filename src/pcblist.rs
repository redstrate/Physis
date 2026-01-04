// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use binrw::{BinRead, binrw};

use crate::{ByteSpan, ReadableFile, common::Platform, pcb::AABB};

/// Collision streaming file, always called `list.pcb`. Despite the name, it's a completely different format to normal PCBs.
#[binrw]
#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<PcbList>();
    }
}
