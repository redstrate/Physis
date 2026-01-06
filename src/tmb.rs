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
pub struct TimelineNode {
    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    tag: String,
    /// Size in bytes, including the tag.
    size: u32,
    #[br(count = size - 8)]
    data: Vec<u8>,
}

/// Timeline binary file, usually with the `.tmb` file extension.
///
/// Contains animation information, and also seen being embedded into SGBs.
#[binrw]
#[brw(magic = b"TMLB")]
#[derive(Debug)]
pub struct Tmb {
    /// In bytes, including this header.
    file_size: u32,
    num_nodes: u32,
    #[br(count = num_nodes)]
    pub nodes: Vec<TimelineNode>,
    // NOTE: there is extra data at the end, presumably it contains the actual animation data?,
}

impl ReadableFile for Tmb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Tmb::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Tmb>();
    }
}
