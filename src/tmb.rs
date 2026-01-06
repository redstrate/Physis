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
pub struct Tmdh {
    unk1: u16,
    unk2: u16,
    unk3: u16,
    unk4: u16,
}

#[binrw]
#[derive(Debug)]
pub struct Tmal {
    offset: u32,
    count: u32,
    // TODO: read u16s
}

#[binrw]
#[derive(Debug)]
pub struct Tmac {
    unk1: u32, // VFXEdit cals this "ability delay" lol
    unk2: u32,
    offset: u32,
    count: u32,
    // TODO: read temp ids
    unk3: u32,
}

#[binrw]
#[derive(Debug)]
pub struct Tmtr {
    offset: u32,
    count: u32,
    // TODO: read temp ids
    unk1: u32,
    unk2: u32,
}

#[binrw]
#[br(import(tag: &str, size: u32))]
#[derive(Debug)]
pub enum TimelineNodeData {
    #[br(pre_assert(tag == "TMDH"))]
    Tmdh(Tmdh),
    #[br(pre_assert(tag == "TMAL"))]
    Tmal(Tmal),
    #[br(pre_assert(tag == "TMAC"))]
    Tmac(Tmac),
    #[br(pre_assert(tag == "TMTR"))]
    Tmtr(Tmtr),
    Unknown(#[br(count = size - 8)] Vec<u8>),
}

#[binrw]
#[derive(Debug)]
pub struct TimelineNode {
    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    tag: String,
    /// Size in bytes, including the tag.
    size: u32,
    #[br(args(&tag, size))]
    #[bw(ignore)] // TODO: suppoort writing
    data: TimelineNodeData,
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
