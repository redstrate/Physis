// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use crate::common_file_operations::read_short_identifier;
use crate::common_file_operations::write_short_identifier;
use binrw::BinRead;
use binrw::BinReaderExt;
use binrw::BinResult;
use binrw::VecArgs;
use binrw::binrw;

/// Reads a 4 byte string.
#[binrw::parser(reader, endian)]
pub(crate) fn read_timeline_list(struct_offset: i64) -> BinResult<Vec<u16>> {
    let offset: i32 = reader.read_type_args(endian, ())?;
    let count: i32 = reader.read_type_args(endian, ())?;

    let pos = reader.stream_position()?;

    reader.seek(SeekFrom::Current(offset as i64 - 8 as i64 - struct_offset))?;

    let list =
        reader.read_type_args(endian, VecArgs::builder().count(count as usize).finalize())?;

    reader.seek(SeekFrom::Start(pos))?;

    Ok(list)
}

#[binrw]
#[derive(Debug)]
pub struct Tmdh {
    unk1: u16,
    unk2: u16,
    duration: u16,
    unk4: u16,
}

/// Timeline Actor List.
#[binrw]
#[derive(Debug)]
pub struct Tmal {
    /// The relevant [Tmac] IDs.
    #[br(parse_with = read_timeline_list, args(0))]
    tmac_ids: Vec<u16>,
}

/// Timeline Actor Control(?)
#[binrw]
#[derive(Debug)]
pub struct Tmac {
    id: u16,
    time: u16,
    unk1: u32, // VFXEdit cals this "ability delay" lol
    unk2: u32,
    #[br(parse_with = read_timeline_list, args(12))]
    list: Vec<u16>,
}

impl Tmac {
    pub const SIZE: usize = 0x0C;
}

#[binrw]
#[derive(Debug)]
pub struct Tmtr {
    id: u16,
    time: u16,
    #[br(parse_with = read_timeline_list, args(4))]
    list: Vec<u16>,
    // TODO: read temp ids
    unk2: u32,
}

#[binrw]
#[derive(Debug)]
pub struct C013 {
    id: u16,
    time: u16,
    duration: i32,
    unk2: i32,
    tmfc_id: i32,
    placement: i32,
}

#[binrw]
#[derive(Debug)]
pub struct Tmfc {
    id: u16,
    time: u16,
    start_offset: i32,
    data_count: i32,
    unk1: i32,
    end_offset: i32,
    unk2: i32,
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
    #[br(pre_assert(tag == "TMFC"))]
    Tmfc(Tmfc),
    #[br(pre_assert(tag == "C013"))]
    C013(C013),
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
