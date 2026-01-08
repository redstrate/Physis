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

#[binrw::parser(reader, endian)]
pub(crate) fn read_timeline_list_2(struct_offset: i64) -> BinResult<Vec<TmfcData>> {
    let offset: i32 = reader.read_type_args(endian, ())?;
    let count: i32 = reader.read_type_args(endian, ())?;

    let pos = reader.stream_position()?;

    reader.seek(SeekFrom::Current(offset as i64 - 4 as i64 - struct_offset))?;

    let mut list: Vec<TmfcData> =
        reader.read_type_args(endian, VecArgs::builder().count(count as usize).finalize())?;

    for data in &mut list {
        data.rows = reader
            .read_type_args(
                endian,
                VecArgs::builder().count(data.row_count as usize).finalize(),
            )
            .unwrap();
    }

    reader.seek(SeekFrom::Start(pos))?;

    Ok(list)
}

#[binrw]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Tmdh {
    unk1: u16,
    unk2: u16,
    pub duration: u16,
    unk4: u16,
}

/// Timeline Actor List.
#[binrw]
#[derive(Debug)]
pub struct Tmal {
    /// The relevant [Tmac] IDs.
    #[br(parse_with = read_timeline_list, args(0))]
    pub tmac_ids: Vec<u16>,
}

/// Timeline Actor Control(?)
#[binrw]
#[derive(Debug)]
pub struct Tmac {
    pub id: u16,
    pub time: u16,
    unk1: u32, // VFXEdit cals this "ability delay" lol
    unk2: u32,
    /// List of Tmtr nodes associated with this actor.
    #[br(parse_with = read_timeline_list, args(12))]
    pub tmtr_ids: Vec<u16>,
}

impl Tmac {
    pub const SIZE: usize = 0x0C;
}

/// Timeline tracks.
#[binrw]
#[derive(Debug)]
pub struct Tmtr {
    pub id: u16,
    pub time: u16, // TODO: is this really the correct name?
    /// List of IDs to CXXX nodes.
    #[br(parse_with = read_timeline_list, args(4))]
    pub animation_ids: Vec<u16>,
    // TODO: read temp ids
    unk2: u32,
}

/// Model animation.
#[binrw]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct C013 {
    pub id: u16,
    pub time: u16,
    pub duration: i32,
    unk2: i32,
    /// ID of the Tmfc node associated with this animation.
    pub tmfc_id: i32,
    placement: i32,
}

/// Timeline F-Curve.
#[binrw]
#[derive(Debug)]
pub struct Tmfc {
    pub id: u16,
    pub time: u16,
    #[br(parse_with = read_timeline_list_2, args(4))]
    pub data: Vec<TmfcData>,
    unk1: i32,
    end_offset: i32,
    unk2: i32,
}

#[binrw]
#[derive(Debug)]
pub enum Attribute {
    #[brw(magic = 64u8)]
    PositionX,
    #[brw(magic = 65u8)]
    PositionY,
    #[brw(magic = 66u8)]
    PositionZ,
    #[brw(magic = 67u8)]
    RotationX,
    #[brw(magic = 68u8)]
    RotationY,
    #[brw(magic = 69u8)]
    RotationZ,
    Unknown(u8),
}

#[binrw]
#[derive(Debug)]
pub struct TmfcData {
    unk1: u32,

    pub attribute: Attribute,
    unk3: u8,
    unk4: u8,
    unk5: u8,

    unk6: u8,
    unk7: u8,
    unk8: u8,
    unk9: u8,

    row_count: u32,
    #[brw(ignore)] // read in read_timeline_list_2
    pub rows: Vec<TmfcRow>,
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct TmfcRow {
    unk1: u32,
    pub time: f32,
    velocity: f32, // According to vfxedit, but I have no idea
    pub value: f32,
    unk4: f32,
    unk5: f32,
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
    pub data: TimelineNodeData,
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
