// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use crate::common_file_operations::read_short_identifier;
use crate::common_file_operations::write_short_identifier;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
struct ScdHeader {
    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    pub file_type: String,

    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    pub sub_type: String,

    version: u32,
    endian_type: u32,
    alignment_bits: u8,
    offset: u16,
    datetime: u64,

    #[br(pad_before = 4)]
    sound_count: u16,
    track_count: u16,
    audio_count: u16,
    number: u16,

    track_offset: u32,
    audio_offset: u32,
    layout_offset: u32,
    routing_offset: u32,
    attribute_offset: u32,

    #[br(pad_after = 2)]
    end_of_file_padding_size: u16,
}

#[derive(Debug)]
pub struct Scd {}

impl ReadableFile for Scd {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        ScdHeader::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        Some(Scd {})
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Scd>();
    }
}
