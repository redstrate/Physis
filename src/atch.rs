// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::common_file_operations::read_string_until_null;
use binrw::binrw;
use binrw::{BinRead, BinWrite};

#[binrw]
#[derive(Debug)]
pub struct AtchEntryState {
    #[br(temp)]
    #[bw(ignore)]
    string_pos: u32,
    #[br(restore_position, seek_before = SeekFrom::Start(string_pos as u64), parse_with = read_string_until_null)]
    #[bw(ignore)]
    pub name: String,
    pub scale: f32,
    pub offset: [f32; 3],
    pub rotation: [f32; 3],
}

/// Attach offset file, usually with the `.atch` file extension.
#[binrw]
#[derive(Debug)]
pub struct Atch {
    #[br(temp)]
    #[bw(calc = entry_names.len() as u16)]
    num_entries: u16,
    #[br(temp)]
    #[bw(calc = states.len() as u16)]
    num_states: u16,
    #[br(count = num_entries)]
    #[bw(ignore)] // TODO: stub
    pub entry_names: Vec<[u8; 4]>, // TODO: use string type
    bitfield: [u64; Self::BITFIELD_SIZE / 8],
    #[br(count = num_entries.saturating_mul(num_states))] // TODO: aggregate by entry
    #[bw(ignore)] // TODO: stub
    pub states: Vec<AtchEntryState>,
}

impl Atch {
    pub const BITFIELD_SIZE: usize = 32;
}

impl ReadableFile for Atch {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);

        Ok(Self::read_options(&mut cursor, endianness, ())?)
    }
}

impl WritableFile for Atch {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
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
        pass_random_invalid::<Atch>();
    }
}
