// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::binrw;
use binrw::{BinRead, BinWrite};

#[binrw]
#[derive(Debug)]
pub struct AtchEntryState {
    string_pos: u32, // TODO: read string
    scale: f32,
    offset: [f32; 3],
    rotation: [f32; 3],
}

/// Attach offset file, usually with the `.atch` file extension.
#[binrw]
#[derive(Debug)]
pub struct Atch {
    num_entries: u16,
    num_states: u16,
    #[br(count = num_entries)]
    #[bw(ignore)] // TODO: stub
    entry_names: Vec<[u8; 4]>, // TODO: use string type
    bitfield: [u64; Self::BITFIELD_SIZE / 8],
    #[br(count = num_entries.saturating_mul(num_states))] // TODO: aggregate by entry
    #[bw(ignore)] // TODO: stub
    states: Vec<AtchEntryState>,
}

impl Atch {
    pub const BITFIELD_SIZE: usize = 32;
}

impl ReadableFile for Atch {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);

        Self::read_options(&mut cursor, endianness, ()).ok()
    }
}

impl WritableFile for Atch {
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
        pass_random_invalid::<Atch>();
    }
}
