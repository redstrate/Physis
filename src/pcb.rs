// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use binrw::BinRead;
use binrw::BinResult;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct PcbResourceHeader {
    pcb_type: u32, // Lumina: 0x0 is resource, 0x1 is list?
    version: u32,  // ClientStructs: 0 is 'legacy', 1/4 are 'normal', rest unsupported
    total_nodes: u32,
    total_polygons: u32,
}

#[binrw::parser(reader)]
fn parse_resource_node_children(
    child1_offset: u32,
    child2_offset: u32,
) -> BinResult<Vec<ResourceNode>> {
    let initial_position = reader.stream_position().unwrap();
    let struct_start = initial_position - ResourceNode::HEADER_SIZE as u64;

    let mut children = Vec::new();
    if child1_offset != 0 {
        reader
            .seek(SeekFrom::Start(struct_start + child1_offset as u64))
            .unwrap();
        children.push(ResourceNode::read_le(reader)?);
    }

    if child2_offset != 0 {
        reader
            .seek(SeekFrom::Start(struct_start + child2_offset as u64))
            .unwrap();
        children.push(ResourceNode::read_le(reader)?);
    }

    Ok(children)
}

/// Transform compressed vertices from 0-65535 to local_bounds.min-local_bounds.max
fn uncompress_vertices(local_bounds: &AABB, vertex: &[u16; 3]) -> [f32; 3] {
    let x_scale = (local_bounds.max[0] - local_bounds.min[0]) / u16::MAX as f32;
    let y_scale = (local_bounds.max[1] - local_bounds.min[1]) / u16::MAX as f32;
    let z_scale = (local_bounds.max[2] - local_bounds.min[2]) / u16::MAX as f32;

    [
        local_bounds.min[0] + x_scale * (vertex[0] as f32),
        local_bounds.min[1] + y_scale * (vertex[1] as f32),
        local_bounds.min[2] + z_scale * (vertex[2] as f32),
    ]
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct ResourceNode {
    // TODO: figure out what these two values are
    magic: u32,   // pretty terrible magic if you ask me, lumina calls it so
    version: u32, // usually 0x0, is this really a version?!

    child1_offset: u32,
    child2_offset: u32,

    /// The bounding box of this node.
    pub local_bounds: AABB,

    num_vert_f16: u16,
    num_polygons: u16,
    #[brw(pad_after = 2)] // padding, supposedly
    num_vert_f32: u16,

    /// The children of this node.
    #[br(parse_with = parse_resource_node_children, args(child1_offset, child2_offset))]
    #[br(restore_position)]
    pub children: Vec<ResourceNode>,

    #[br(count = num_vert_f32)]
    f32_vertices: Vec<[f32; 3]>,
    #[br(count = num_vert_f16)]
    #[bw(ignore)]
    f16_vertices: Vec<[u16; 3]>,

    /// This node's vertices.
    #[br(calc = f32_vertices.clone().into_iter().chain(f16_vertices.iter().map(|vec| uncompress_vertices(&local_bounds, vec))).collect())]
    #[bw(ignore)]
    pub vertices: Vec<[f32; 3]>,

    /// This node's polygons, which include index data.
    #[br(count = num_polygons)]
    pub polygons: Vec<Polygon>,
}

impl ResourceNode {
    pub const HEADER_SIZE: usize = 0x30;
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct AABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Polygon {
    #[brw(pad_after = 1)] // padding
    pub vertex_indices: [u8; 3],
    pub material: u64,
}

/// Player collision binary file, usually with the `.pcb` file extension.
///
/// Contains a tree of polygons that makes up a collision mesh.
#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct Pcb {
    header: PcbResourceHeader,
    /// The root node of this PCB.
    pub root_node: ResourceNode,
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
