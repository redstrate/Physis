// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // binrw :-(

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use crate::common_file_operations::read_string;
use crate::common_file_operations::read_string_until_null;
use crate::common_file_operations::write_string;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(import(name: &str))]
#[brw(little)]
pub enum NodeData {
    #[br(pre_assert(name == "CTRL"))]
    CTRL(CTRLNode),
    #[br(pre_assert(name == "CTIS"))]
    CTIS(CTISNode),
    #[br(pre_assert(name == "CTDS"))]
    CTDS(CTDSNode),
    #[br(pre_assert(name == "CTTL"))]
    CTTL(CTTLNode),
    Unknown,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct CutsceneNode {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = write_string)]
    #[br(map = read_string)]
    pub name: String,
    /// In bytes, the size of this node *including* the name.
    size: u32,

    /// Offset starting from the beginning of this field to the node information.
    data_offset: u32,

    #[br(seek_before = SeekFrom::Current(data_offset as i64 - 4))]
    #[br(restore_position)]
    #[br(args(&name))]
    pub node_data: NodeData,

    /// Size of the node's data *including* the node information.
    data_size: u32,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct StringNode {
    #[br(temp)]
    #[bw(ignore)]
    offset: u32,

    // TODO: we may be using offset wrong, unsure
    #[br(seek_before = SeekFrom::Current(offset as i64 - 4))]
    #[br(restore_position, parse_with = read_string_until_null)]
    #[bw(ignore)]
    pub value: String,

    #[br(temp)]
    #[bw(ignore)]
    unk1: u32, // Seems to be either 255 or 0
}

/// Cutscene binary file, usually with the `.cutb` file extension.
///
/// Describes animated cutscenes to be played in-game.
#[binrw]
#[brw(magic = b"CUTB")]
#[derive(Debug)]
#[brw(little)]
pub struct Cutscene {
    /// In bytes, including the header and the magic.
    size_of_file: u32,
    num_nodes: u32,
    #[br(count = num_nodes)]
    pub nodes: Vec<CutsceneNode>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct CTRLNode {
    /// In bytes, the size of the node information *including* this field.
    size: u32, // number of following u32s
    num_string_nodes: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    #[br(count = num_string_nodes)]
    string_nodes: Vec<StringNode>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct CTISNode {
    /// In bytes, the size of the node information *including* this field.
    size: u32, // number of following u32s
    unk1: u32,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct CTDSNode {
    /// In bytes, the size of the node information *including* this field.
    size: u32, // number of following u32s
    unk1: [u32; 15],
    num_entries: u32,
    unk2: u32,
    /// From the beginning of this node.
    offset_to_level: u32,
    #[br(seek_before = SeekFrom::Current(offset_to_level as i64 - 76))] // 76 is the offset within this node, yes i know it's bad
    #[br(restore_position, parse_with = read_string_until_null)]
    #[bw(ignore)]
    level_name: String,
    unk3: [u32; 2], // seems to be empty
    #[br(count = num_entries)]
    entries: Vec<u64>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct CTTLNode {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = write_string)]
    #[br(map = read_string)]
    name: String,
    size: u32,
    node_count: u32,
    // TODO: unsure if this is really a count
    #[br(count = node_count)]
    node: Vec<TimelineNode>,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct TimelineNode {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = write_string)]
    #[br(map = read_string)]
    name: String,
    /// The size of this whole struct
    size: u32,
    // TODO: parse this data
    #[br(count = size - 8)]
    data: Vec<u8>,
}

impl Cutscene {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Cutscene> {
        let mut cursor = Cursor::new(buffer);
        Cutscene::read(&mut cursor).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::read, path::PathBuf};

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Cutscene::from_existing(&read(d).unwrap());
    }
}
