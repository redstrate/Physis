// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::{BinReaderExt, BinResult, BinWrite, binrw};

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapStringFromPointer, StringHeap},
};

#[binrw::writer(writer, endian)]
pub(crate) fn write_scns(scns: &Vec<ScnSection>, string_heap: &mut StringHeap) -> BinResult<()> {
    for scn in scns {
        scn.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

/// SCN1 section used in LVBs and SGBs.
#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
#[brw(magic = b"SCN1")]
pub struct ScnSection {
    /// Size of this header. Should be equal to `ScnHeader::SIZE`.
    total_size: u32,
    /// Offset to FileLayerGroupHeader[NumEmbeddedLayerGroups].
    pub(crate) offset_layer_groups: i32,
    /// Number of embedded layer groups.
    pub(crate) num_layer_groups: i32,
    /// Offset to FileSceneGeneral.
    offset_general: i32,
    /// Offset to FileSceneFilterList.
    offset_filters: i32,
    /// Offset to FileSceneTimelineList.
    offset_timelines: i32,
    /// offset to a list of path offsets (ints)
    offset_layer_group_resources: i32,
    num_layer_group_resources: i32,
    unk2: i32,
    offset_unk1: i32, // Points to 5 bytes of data
    unk4: i32,
    unk5: i32,
    housing_offset: i32, // According to Lumina?
    unk7: i32,
    unk8: i32,
    unk9: i32,
    unk10: i32,
    offset_unk2: i32, // Points to 39 bytes of data
    offset_unk3: i32, // Points to 64 bytes of data

    #[br(seek_before = SeekFrom::Current(offset_general as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    #[brw(args(string_heap))]
    pub general: ScnGeneralSection,

    #[br(seek_before = SeekFrom::Current(offset_filters as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    #[brw(args(string_heap))]
    pub filters: ScnFiltersSection,

    #[br(seek_before = SeekFrom::Current(offset_timelines as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    pub timelines: ScnTimelinesSection,

    #[br(count = num_layer_group_resources)]
    #[br(seek_before = SeekFrom::Current(offset_layer_group_resources as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    offset_path_layer_group_resources: Vec<i32>,

    #[br(parse_with = strings_from_offsets)]
    #[br(args(&offset_path_layer_group_resources))]
    #[br(restore_position)]
    #[br(seek_before = SeekFrom::Current(offset_layer_group_resources as i64 - ScnSection::SIZE as i64))]
    #[bw(ignore)] // TODO: support
    pub lgb_paths: Vec<String>,

    #[br(seek_before = SeekFrom::Current(offset_unk1 as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    unk1: ScnUnknown1Section,

    #[br(seek_before = SeekFrom::Current(offset_unk2 as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    unk2_section: ScnUnknown2Section,

    #[br(seek_before = SeekFrom::Current(offset_unk3 as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    unk3: ScnUnknown3Section,
}

impl ScnSection {
    pub const SIZE: usize = 0x48;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct ScnEnvSpace {
    #[br(temp)]
    #[bw(ignore)]
    heap_pointer: HeapPointer,

    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub env_path: HeapStringFromPointer,

    unk1: i32,
    unk2: i32,

    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub essb_path: HeapStringFromPointer,

    // TODO: I have no idea, but there's 8 extra bytes unaccounted for here. Probably a mistake elsewhere.
    #[br(restore_position)]
    unk: u64,
}

impl ScnEnvSpace {
    pub const SIZE: usize = 0x10;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct ScnGeneralSection {
    #[br(temp)]
    #[bw(ignore)]
    heap_pointer: HeapPointer,

    #[br(map = read_bool_from::<i32>)]
    #[bw(map = write_bool_as::<i32>)]
    pub have_layer_groups: bool, // TODO: this is probably not what is according to Sapphire?

    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub bg_path: HeapStringFromPointer,

    offset_env_spaces: i32,
    num_env_spaces: i32,

    unk1: i32,

    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub svb_path: HeapStringFromPointer,

    unk2: i32,
    unk3: i32,
    unk4: i32,
    unk5: i32,
    unk6: i32,
    unk7: i32,
    unk8: i32, // points to 4 bytes in the string heap

    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub lcb_path: HeapStringFromPointer,

    unk10: i32,
    unk11: i32,
    unk12: i32,
    unk13: i32,
    unk14: i32,
    unk15: i32,
    unk16: i32,

    #[br(map = read_bool_from::<i32>)]
    #[bw(map = write_bool_as::<i32>)]
    pub have_lcbuw: bool,

    #[br(count = num_env_spaces)]
    #[br(seek_before = SeekFrom::Current(offset_env_spaces as i64 - ScnGeneralSection::SIZE as i64))]
    #[br(restore_position)]
    #[br(args { inner: (string_heap,) })]
    #[bw(write_with = write_env_spaces, args(string_heap))]
    pub env_spaces: Vec<ScnEnvSpace>,
}

#[binrw::writer(writer, endian)]
pub fn write_env_spaces(scns: &Vec<ScnEnvSpace>, string_heap: &mut StringHeap) -> BinResult<()> {
    for scn in scns {
        scn.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

impl ScnGeneralSection {
    pub const SIZE: usize = 0x58;
}

#[binrw]
#[derive(Debug)]
pub struct ScnTimelinesSection {
    offset_entries: i32,
    num_entries: i32,
}

impl ScnTimelinesSection {
    pub const SIZE: usize = 0x8;
}

// TODO: definitely not correct
#[binrw]
#[derive(Debug)]
pub struct ScnUnknown1Section {
    unk: [u8; 5],
}

// TODO: definitely not correct
#[binrw]
#[derive(Debug)]
pub struct ScnUnknown2Section {
    unk: [u8; 39],
}

// TODO: definitely not correct
#[binrw]
#[derive(Debug)]
pub struct ScnUnknown3Section {
    unk: [u8; 64],
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct ScnFiltersSection {
    filter_offset: i32,
    filter_count: i32,

    #[br(seek_before = SeekFrom::Current(filter_offset as i64 - ScnFiltersSection::SIZE as i64))]
    #[br(count = filter_count, restore_position, args { inner: (string_heap,) })]
    #[bw(write_with = write_filters, args(string_heap))]
    pub filters: Vec<ScnFilter>,
}

#[binrw::writer(writer, endian)]
pub fn write_filters(scns: &Vec<ScnFilter>, string_heap: &mut StringHeap) -> BinResult<()> {
    for scn in scns {
        scn.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

impl ScnFiltersSection {
    pub const SIZE: usize = 0x8;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct ScnFilter {
    #[br(temp)]
    #[bw(ignore)]
    heap_pointer: HeapPointer,

    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub nvm_path: HeapStringFromPointer,

    unk1: i32,
    unk2: i32,
    unk3: i32,

    /// Refers to a row in the TerritoryType Excel sheet.
    pub territory_type_id: i32,

    unk5: i32,

    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub nvx_path: HeapStringFromPointer,
}

impl ScnFilter {
    pub const SIZE: usize = 0x1C;
}

#[binrw::parser(reader)]
fn strings_from_offsets(offsets: &Vec<i32>) -> BinResult<Vec<String>> {
    let base_offset = reader.stream_position()?;

    let mut strings: Vec<String> = vec![];

    for offset in offsets {
        let string_offset = *offset as u64;

        let mut string = String::new();

        reader.seek(SeekFrom::Start(base_offset + string_offset))?;
        let mut next_char = reader.read_le::<u8>().unwrap() as char;
        while next_char != '\0' {
            string.push(next_char);
            next_char = reader.read_le::<u8>().unwrap() as char;
        }

        strings.push(string);
    }

    Ok(strings)
}
