// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;

use crate::AABB;
use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::binrw;

#[binrw::parser(reader, endian)]
fn parse_resource_node_children(child1_offset: i32, child2_offset: i32) -> BinResult<Vec<Node>> {
    let initial_position = reader.stream_position()?;
    let struct_start = initial_position - Node::HEADER_SIZE as u64;

    let mut children = Vec::new();
    if child1_offset != 0 {
        reader.seek(SeekFrom::Start(struct_start + child1_offset as u64))?;
        children.push(Node::read_options(reader, endian, ())?);
    }

    if child2_offset != 0 {
        reader.seek(SeekFrom::Start(struct_start + child2_offset as u64))?;
        children.push(Node::read_options(reader, endian, ())?);
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
pub struct Node {
    // TODO: figure out what these two values are
    magic: u32,   // pretty terrible magic if you ask me, lumina calls it so
    version: u32, // usually 0x0, is this really a version?!

    child1_offset: i32,
    child2_offset: i32,

    /// The bounding box of this node.
    pub local_bounds: AABB,

    #[bw(calc = 0)]
    num_vert_f16: u16,
    #[bw(calc = polygons.len() as u16)]
    num_polygons: u16,
    #[brw(pad_after = 2)] // padding, supposedly
    #[bw(calc = vertices.len() as u16)]
    num_vert_f32: u16,

    /// The children of this node.
    #[br(parse_with = parse_resource_node_children, args(child1_offset, child2_offset))]
    #[br(restore_position)]
    pub children: Vec<Node>,

    #[bw(calc = vertices.clone())]
    #[br(count = num_vert_f32)]
    f32_vertices: Vec<[f32; 3]>,
    #[br(count = num_vert_f16)]
    #[bw(calc = Vec::new())]
    f16_vertices: Vec<[u16; 3]>,

    /// This node's vertices.
    #[br(calc = f32_vertices.clone().into_iter().chain(f16_vertices.iter().map(|vec| uncompress_vertices(&local_bounds, vec))).collect())]
    #[bw(ignore)]
    pub vertices: Vec<[f32; 3]>,

    /// This node's polygons, which include index data.
    #[br(count = num_polygons)]
    pub polygons: Vec<Polygon>,
}

impl Node {
    pub(crate) const HEADER_SIZE: usize = 0x30;

    fn child_node_count(&self) -> usize {
        let mut count = self.children.len();
        for child in &self.children {
            count += child.child_node_count();
        }

        count
    }

    fn polygon_count(&self) -> usize {
        let mut count = self.polygons.len();
        for child in &self.children {
            count += child.polygon_count();
        }

        count
    }
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
pub enum Pcb {
    Mesh(Node),
    List(List),
}

#[binrw]
#[derive(Debug, Default)]
pub struct List {
    /// How many entries are in the file.
    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    entry_count: u32,
    #[brw(pad_after = 4)] // empty padding
    pub bounds: AABB,
    #[br(count = entry_count)]
    pub entries: Vec<ListEntry>,
}

/// Represents a PCB file in the same directory, with the filename `tr{mesh_id}.pcb`.
#[binrw]
#[derive(Debug)]
pub struct ListEntry {
    pub mesh_id: u32,
    #[brw(pad_after = 4)] // empty padding
    pub bounds: AABB,
}

impl ReadableFile for Pcb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);

        let entry_count = u32::read_options(&mut cursor, platform.endianness(), ())?;
        if entry_count == 0 {
            let _ = u32::read_options(&mut cursor, platform.endianness(), ())?; // version
            let _ = u32::read_options(&mut cursor, platform.endianness(), ())?; // total nodes
            let _ = u32::read_options(&mut cursor, platform.endianness(), ())?; // total poylgons

            Ok(Self::Mesh(Node::read_options(
                &mut cursor,
                platform.endianness(),
                (),
            )?))
        } else {
            cursor.seek(SeekFrom::Current(-(std::mem::size_of::<u32>() as i64)))?; // List will read the entry count again!

            Ok(Self::List(List::read_options(
                &mut cursor,
                platform.endianness(),
                (),
            )?))
        }
    }
}

impl WritableFile for Pcb {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);

            if let Pcb::Mesh(node) = self {
                let entry_count = 0u32;
                entry_count.write_options(&mut cursor, platform.endianness(), ())?;

                let version = 1u32;
                version.write_options(&mut cursor, platform.endianness(), ())?;

                let total_nodes = node.child_node_count() as u32;
                total_nodes.write_options(&mut cursor, platform.endianness(), ())?;

                let total_polygons = node.polygon_count() as u32;
                total_polygons.write_options(&mut cursor, platform.endianness(), ())?;
            }

            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

impl Pcb {
    /// Creates a very simple collision mesh from a list of vertices and polygons.
    pub fn new_from_vertices(vertices: &[[f32; 3]], polygons: &[Polygon]) -> Self {
        let mut local_bounds = AABB {
            min: [f32::INFINITY; 3],
            max: [-f32::INFINITY; 3],
        };
        for vertex in vertices {
            local_bounds.min[0] = local_bounds.min[0].min(vertex[0]);
            local_bounds.min[1] = local_bounds.min[1].min(vertex[1]);
            local_bounds.min[2] = local_bounds.min[2].min(vertex[2]);

            local_bounds.max[0] = local_bounds.max[0].max(vertex[0]);
            local_bounds.max[1] = local_bounds.max[1].max(vertex[1]);
            local_bounds.max[2] = local_bounds.max[2].max(vertex[2]);
        }

        Self::Mesh(Node {
            magic: 0,
            version: 0,
            child1_offset: 0,
            child2_offset: 0,
            local_bounds,
            children: Vec::new(),
            vertices: vertices.to_vec(),
            polygons: polygons.to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::read, path::PathBuf};

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Pcb>();
    }

    #[test]
    fn test_write_triangle() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("triangle.pcb");

        let empty_pcb = &read(d).unwrap();
        let pcb = Pcb::new_from_vertices(
            &[
                [-400.00006, -20.0, -800.0],
                [-400.00006, -20.0, -400.0],
                [-3.0517578e-5, -10.0, -400.0],
            ],
            &[Polygon {
                vertex_indices: [0, 1, 2],
                material: 28672,
            }],
        );

        assert_eq!(*empty_pcb, pcb.write_to_buffer(Platform::Win32).unwrap());
    }

    #[test]
    fn test_write_empty() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty_list.pcb");

        let empty_pcb = &read(d).unwrap();
        let pcb = Pcb::List(List::default());

        assert_eq!(*empty_pcb, pcb.write_to_buffer(Platform::Win32).unwrap());
    }
}
