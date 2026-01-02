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
struct SgbHeader {
    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    pub identifier: String,

    file_size: i32,
    total_chunk_count: i32,
}

/// Shared group binary file, usually with the `.sgb` file extension.
///
/// This is basically a "prefab".
#[derive(Debug)]
pub struct Sgb {}

impl ReadableFile for Sgb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        SgbHeader::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        Some(Sgb {})
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Sgb>();
    }
}
