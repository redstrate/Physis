// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use binrw::BinRead;
use binrw::BinResult;
use binrw::binrw;
use binrw::helpers::until_eof;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct PcbResourceHeader {
    magic: u32,   // pretty terrible magic if you ask me, lumina calls it so but it's just 0000
    version: u32, // usually 0x1?
    total_nodes: u32,
    total_polygons: u32,
}

// TODO: this is adapted from lumina and could probably be implemented better
#[binrw::parser(reader)]
fn parse_resource_node_children(
    group_length: u32,
    header_skip: u32,
) -> BinResult<Vec<ResourceNode>> {
    if group_length == 0 {
        return Ok(Vec::default());
    }

    assert!(header_skip > 0);

    let mut children = Vec::new();
    let initial_position = reader.stream_position().unwrap() - header_skip as u64;
    let final_position = initial_position + group_length as u64;

    while reader.stream_position().unwrap() + (header_skip as u64) < final_position {
        children.push(ResourceNode::read_le(reader).unwrap());
    }

    reader.seek(SeekFrom::Start(final_position)).unwrap();

    Ok(children)
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct ResourceNode {
    magic: u32,   // pretty terrible magic if you ask me, lumina calls it so
    version: u32, // usually 0x0, is this really a version?!
    header_skip: u32,
    group_length: u32,
    pub bounding_box: BoundingBox,

    num_vert_f16: u16,
    num_polygons: u16,
    #[brw(pad_after = 2)] // padding
    num_vert_f32: u16,

    #[br(parse_with = parse_resource_node_children, args(group_length, header_skip))]
    pub children: Vec<ResourceNode>,

    // TODO: combine these
    #[br(count = num_vert_f32)]
    pub f32_vertices: Vec<[f32; 3]>,
    #[br(count = num_vert_f16)]
    pub f16_vertices: Vec<[u16; 3]>,
    #[br(count = num_polygons)]
    pub polygons: Vec<Polygon>,
}

// TODO: de-duplicate with MDL?
#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct BoundingBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct Polygon {
    pub vertex_indices: [u8; 3],
    #[brw(pad_before = 2, pad_after = 5)] // padding
    unk1: u16,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct Pcb {
    header: PcbResourceHeader,
    // NOTE: this is technically wrong, we should be counting each node until we get total_nodes but im lazy
    #[br(parse_with = until_eof)]
    pub children: Vec<ResourceNode>,
}

impl Pcb {
    /// Reads an existing PCB file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Pcb::read(&mut cursor).ok()
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
        Pcb::from_existing(&read(d).unwrap());
    }
}
