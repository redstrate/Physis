// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(dead_code)]

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use crate::common_file_operations::read_bool_from;
use crate::common_file_operations::read_string_until_null;
use binrw::BinRead;
use binrw::BinReaderExt;
use binrw::BinResult;
use binrw::binread;

#[binread]
#[derive(Debug)]
#[brw(little)]
#[brw(magic = b"LVB1")]
pub struct Lvb {
    /// Including this header
    pub file_size: u32,
    /// Number of Scn's
    #[br(temp)]
    #[bw(calc = scns.len() as u32)]
    scn_count: u32,
    #[br(count = scn_count)]
    pub scns: Vec<Scn>,
}

#[binread]
#[derive(Debug)]
#[brw(little)]
#[brw(magic = b"SCN1")]
pub struct Scn {
    total_size: u32,
    pub header: ScnHeader,
    #[br(seek_before = SeekFrom::Current(header.offset_general as i64 - ScnHeader::SIZE as i64))]
    #[br(restore_position)]
    pub general: ScnGeneralSection,
    #[br(seek_before = SeekFrom::Current(header.offset_filters as i64 - ScnHeader::SIZE as i64))]
    #[br(restore_position)]
    pub unk3: ScnUnknown3Section,
    #[br(seek_before = SeekFrom::Current(header.offset_unk1 as i64 - ScnHeader::SIZE as i64))]
    #[br(restore_position)]
    pub unk1: ScnUnknown1Section,
    #[br(seek_before = SeekFrom::Current(header.offset_unk2 as i64 - ScnHeader::SIZE as i64))]
    #[br(restore_position)]
    pub unk2: ScnUnknown2Section,
}

#[binrw::parser(reader)]
pub(crate) fn strings_from_offsets(offsets: &Vec<i32>) -> BinResult<Vec<String>> {
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

#[binread]
#[derive(Debug)]
#[brw(little)]
pub struct ScnHeader {
    /// offset to FileLayerGroupHeader[NumEmbeddedLayerGroups]
    offset_embedded_layer_groups: i32,
    num_embedded_layer_groups: i32,
    /// offset to FileSceneGeneral
    offset_general: i32,
    /// offset to FileSceneFilterList
    offset_filters: i32,
    offset_unk1: i32,
    /// offset to a list of path offsets (ints)
    offset_layer_group_resources: i32,
    num_layer_group_resources: i32,
    unk2: i32,
    offset_unk2: i32,
    unk4: i32,
    unk5: i32,
    unk6: i32,
    unk7: i32,
    unk8: i32,
    unk9: i32,
    unk10: i32,

    #[br(count = num_layer_group_resources)]
    #[br(seek_before = SeekFrom::Current(offset_layer_group_resources as i64 - ScnHeader::SIZE as i64))]
    #[br(restore_position)]
    offset_path_layer_group_resources: Vec<i32>,

    #[br(parse_with = strings_from_offsets)]
    #[br(args(&offset_path_layer_group_resources))]
    #[br(restore_position)]
    #[br(seek_before = SeekFrom::Current(offset_layer_group_resources as i64 - ScnHeader::SIZE as i64))]
    pub path_layer_group_resources: Vec<String>,
}

impl ScnHeader {
    pub const SIZE: usize = 0x40;
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ScnGeneralSection {
    #[br(map = read_bool_from::<i32>)]
    pub have_layer_groups: bool, // TODO: this is probably not what is according to Sapphire?
    offset_path_terrain: i32,
    offset_env_spaces: i32,
    num_env_spaces: i32,
    unk1: i32,
    offset_path_sky_visibility: i32,
    unk2: i32,
    unk3: i32,
    unk4: i32,
    unk5: i32,
    unk6: i32,
    unk7: i32,
    unk8: i32,
    offset_path_lcb: i32,
    unk10: i32,
    unk11: i32,
    unk12: i32,
    unk13: i32,
    unk14: i32,
    unk15: i32,
    unk16: i32,
    #[br(map = read_bool_from::<i32>)]
    pub have_lcbuw: bool,

    #[br(seek_before = SeekFrom::Current(offset_path_terrain as i64 - ScnGeneralSection::SIZE as i64))]
    #[br(restore_position, parse_with = read_string_until_null)]
    pub path_terrain: String,

    #[br(seek_before = SeekFrom::Current(offset_path_sky_visibility as i64 - ScnGeneralSection::SIZE as i64))]
    #[br(restore_position, parse_with = read_string_until_null)]
    pub path_sky_visibility: String,

    #[br(seek_before = SeekFrom::Current(offset_path_lcb as i64 - ScnGeneralSection::SIZE as i64))]
    #[br(restore_position, parse_with = read_string_until_null)]
    pub path_lcb: String,
}

impl ScnGeneralSection {
    pub const SIZE: usize = 0x58;
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ScnUnknown1Section {
    unk1: i32,
    unk2: i32,
}

impl ScnUnknown1Section {
    pub const SIZE: usize = 0x8;
}

// TODO: definitely not correct
#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ScnUnknown2Section {
    unk1: i32,
    unk2: i32,
}

impl ScnUnknown2Section {
    pub const SIZE: usize = 0x8;
}

// TODO: definitely not correct
#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ScnUnknown3Section {
    layer_sets_offset: i32,
    layer_sets_count: i32,

    #[br(seek_before = SeekFrom::Current(layer_sets_offset as i64 - ScnUnknown3Section::SIZE as i64))]
    #[br(count = layer_sets_count, restore_position)]
    pub unk2: Vec<ScnUnknown4Section>,
}

impl ScnUnknown3Section {
    pub const SIZE: usize = 0x8;
}

// TODO: definitely not correct
#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ScnUnknown4Section {
    nvm_path_offset: i32,
    unk1: i32,
    unk2: i32,
    unk3: i32,
    unk4: i32,
    unk5: i32,
    nvx_path_offset: i32,

    #[br(seek_before = SeekFrom::Current(nvm_path_offset as i64 - ScnUnknown4Section::SIZE as i64))]
    #[br(restore_position, parse_with = read_string_until_null)]
    pub path_nvm: String,

    #[br(seek_before = SeekFrom::Current(nvx_path_offset as i64 - ScnUnknown4Section::SIZE as i64))]
    #[br(restore_position, parse_with = read_string_until_null)]
    pub path_nvx: String,
}

impl ScnUnknown4Section {
    pub const SIZE: usize = 0x1C;
}

impl Lvb {
    /// Reads an existing UWB file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Lvb::read(&mut cursor).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Lvb::from_existing(&read(d).unwrap());
    }
}
