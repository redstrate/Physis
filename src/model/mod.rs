// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::unnecessary_fallible_conversions)] // This wrongly trips on binrw code

mod file_operations;

pub mod vertex_declarations;

use std::io::{Cursor, Seek, SeekFrom};
use std::mem::size_of;

use binrw::{BinRead, VecArgs};
use binrw::{BinReaderExt, BinResult};
use binrw::{BinWrite, BinWriterExt, binrw};
use bitflags::bitflags;

use crate::common::Platform;
use crate::common_file_operations::{read_bool_from, write_bool_as};
use crate::{ByteBuffer, ByteSpan, ReadableFile, WritableFile};
use vertex_declarations::{
    VERTEX_ELEMENT_SIZE, VertexDeclaration, VertexType, VertexUsage, vertex_element_parser,
    vertex_element_writer,
};

pub const NUM_VERTICES: u32 = 17;

#[binrw]
#[derive(Debug, Clone, PartialEq)]
pub struct ModelFileHeader {
    pub version: u32,

    pub stack_size: u32,
    pub runtime_size: u32,

    pub vertex_declaration_count: u16,
    pub material_count: u16,

    pub vertex_offsets: [u32; 3],
    pub index_offsets: [u32; 3],
    pub vertex_buffer_size: [u32; 3],
    pub index_buffer_size: [u32; 3],

    pub lod_count: u8,

    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub index_buffer_streaming_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)]
    pub has_edge_geometry: bool,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Copy)]
struct ModelFlags1(u8);

bitflags! {
    impl ModelFlags1: u8 {
        const None = 0x0;
        const ShadowDisabled = 0x01;
        const LightShadowDisabled = 0x02;
        const WavingAnimationDisabled = 0x04;
        const LightingReflectionEnabled = 0x08;
        const RainOcclusionEnabled = 0x20;
        const SnowOcclusionEnabled = 0x40;
        const DustOcclusionEnabled = 0x80;
    }
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Copy)]
struct ModelFlags2(u8);

bitflags! {
    impl ModelFlags2: u8 {
        const None = 0x0;
        const Unknown3 = 0x01;
        const EdgeGeometryEnabled = 0x02;
        const ForceLodRangeEnabled = 0x04;
        const ShadowMaskEnabled = 0x08;
        const ExtraLodEnabled = 0x10;
        const EnableForceNonResident = 0x20;
        const BgUvScrollEnabled = 0x40;
        const Unknown2 = 0x80;
    }
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[br(import { vertex_declaration_count: u16 })]
#[allow(dead_code)]
pub struct ModelHeader {
    #[br(args(vertex_declaration_count), parse_with = vertex_element_parser)]
    #[bw(write_with = vertex_element_writer)]
    pub vertex_declarations: Vec<VertexDeclaration>,

    #[brw(pad_after = 2)]
    string_count: u16,
    string_size: u32,

    #[br(count = string_size)]
    strings: Vec<u8>,

    radius: f32,

    mesh_count: u16,
    attribute_count: u16,
    submesh_count: u16,
    material_count: u16,

    bone_count: u16,
    bone_table_count: u16,

    shape_count: u16,
    shape_mesh_count: u16,
    shape_value_count: u16,

    lod_count: u8,

    flags1: ModelFlags1,

    element_id_count: u16,
    terrain_shadow_mesh_count: u8,

    flags2: ModelFlags2,

    model_clip_out_of_distance: f32,
    shadow_clip_out_of_distance: f32,

    furniture_part_bounding_box_count: u16,
    terrain_shadow_submesh_count: u16,

    flags3: u8,

    bg_change_material_index: u8,
    bg_crest_change_material_index: u8,

    neck_morph_table_size: u8,
    bone_set_size: u16,

    unknown13: u16,
    patch_72_table_size: u16,
    unknown15: u16,
    unknown16: u16,
    unknown17: u16,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct MeshLod {
    mesh_index: u16,
    mesh_count: u16,

    model_lod_range: f32,
    texture_lod_range: f32,

    water_mesh_index: u16,
    water_mesh_count: u16,

    shadow_mesh_index: u16,
    shadow_mesh_count: u16,

    terrain_shadow_mesh_index: u16,
    terrain_shadow_mesh_count: u16,

    vertical_fog_mesh_index: u16,
    vertical_fog_mesh_count: u16,

    // unused on win32 according to lumina devs
    edge_geometry_size: u32,
    edge_geometry_data_offset: u32,

    unk6: u32,
    unk7: u32,

    vertex_buffer_size: u32,
    index_buffer_size: u32,
    vertex_data_offset: u32,
    index_data_offset: u32,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct Mesh {
    #[brw(pad_after = 2)]
    vertex_count: u16,
    index_count: u32,

    material_index: u16,
    submesh_index: u16,
    submesh_count: u16,

    bone_table_index: u16,
    start_index: u32,

    vertex_buffer_offsets: [u32; 3],
    vertex_buffer_strides: [u8; 3],

    vertex_stream_count: u8,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct Submesh {
    index_offset: u32,
    index_count: u32,

    attribute_index_mask: u32,

    bone_start_index: u16,
    bone_count: u16,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct BoneTable {
    bone_indices: [u16; 64],

    #[brw(pad_after = 3)]
    bone_count: u8,
}

#[derive(Debug, Clone, PartialEq)]
struct BoneTableV2 {
    bone_indices: Vec<u16>,
}

#[derive(Default, Debug, Clone, PartialEq)]
struct BoneTablesV2 {
    offset_counts: Vec<(u16, u16)>,
    tables: Vec<BoneTableV2>,
}

fn pad_to_alignment<T: Seek>(reader: &mut T) -> BinResult<()> {
    let position = reader.stream_position().unwrap() as i64;

    // pad to 4 byte alignment
    let padding = if position % 4 == 0 {
        0
    } else {
        (0 - (position % 4) + 4) % 4
    };
    reader.seek(SeekFrom::Current(padding))?;

    Ok(())
}

#[binrw::parser(reader, endian)]
fn read_bone_tables_v2(bone_table_count: u16) -> BinResult<BoneTablesV2> {
    let offset_counts: Vec<(u16, u16)> = reader.read_type_args(
        endian,
        VecArgs::builder()
            .count(bone_table_count as usize)
            .finalize(),
    )?;

    let mut tables = Vec::new();
    for (_, count) in &offset_counts {
        let bone_indices =
            reader.read_type_args(endian, VecArgs::builder().count(*count as usize).finalize())?;
        tables.push(BoneTableV2 { bone_indices });
        pad_to_alignment(reader)?;
    }

    Ok(BoneTablesV2 {
        offset_counts,
        tables,
    })
}

#[binrw::writer(writer, endian)]
fn write_bone_tables_v2(bone_tables_v2: &BoneTablesV2) -> BinResult<()> {
    bone_tables_v2
        .offset_counts
        .write_options(writer, endian, ())?;
    for table in &bone_tables_v2.tables {
        table.bone_indices.write_options(writer, endian, ())?;
        pad_to_alignment(writer)?;
    }

    Ok(())
}

#[binrw]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
struct BoundingBox {
    min: [f32; 4],
    max: [f32; 4],
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct TerrainShadowMesh {
    index_count: u32,
    start_index: u32,
    vertex_buffer_offset: u32,
    vertex_count: u16,
    submesh_index: u16,
    submesh_count: u16,
    vertex_buffer_stride: u8,
    padding: u8,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct TerrainShadowSubmesh {
    index_offset: u32,
    index_count: u32,
    unknown1: u16,
    unknown2: u16,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct ShapeStruct {
    string_offset: u32,
    shape_mesh_start_index: [u16; 3],
    shape_mesh_count: [u16; 3],
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct ShapeMesh {
    mesh_index_offset: u32,
    shape_value_count: u32,
    shape_value_offset: u32,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct ShapeValue {
    base_indices_index: u16,
    replacing_vertex_index: u16,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
#[brw(import(file_header: &ModelFileHeader))]
pub struct ModelData {
    #[br(args { vertex_declaration_count: file_header.vertex_declaration_count })]
    pub header: ModelHeader,

    #[br(count = header.element_id_count)]
    element_ids: Vec<ElementId>,

    #[br(count = 3)]
    lods: Vec<MeshLod>,

    #[br(count = header.mesh_count)]
    meshes: Vec<Mesh>,

    #[br(count = header.attribute_count)]
    attribute_name_offsets: Vec<u32>,

    #[br(count = header.terrain_shadow_mesh_count)]
    terrain_shadow_meshes: Vec<TerrainShadowMesh>,

    #[br(count = header.submesh_count)]
    submeshes: Vec<Submesh>,

    #[br(count = header.terrain_shadow_submesh_count)]
    terrain_shadow_submeshes: Vec<TerrainShadowSubmesh>,

    #[br(count = header.material_count)]
    material_name_offsets: Vec<u32>,

    #[br(count = header.bone_count)]
    bone_name_offsets: Vec<u32>,

    #[br(count = header.bone_table_count)]
    #[brw(if(file_header.version <= 0x1000005))]
    bone_tables: Vec<BoneTable>,

    #[brw(if(file_header.version >= 0x1000006))]
    #[br(parse_with = read_bone_tables_v2, args(header.bone_table_count))]
    #[bw(write_with = write_bone_tables_v2)]
    bone_tables_v2: BoneTablesV2,

    #[br(count = header.shape_count)]
    shapes: Vec<ShapeStruct>,

    #[br(count = header.shape_mesh_count)]
    shape_meshes: Vec<ShapeMesh>,

    #[br(count = header.shape_value_count)]
    shape_values: Vec<ShapeValue>,

    submesh_bone_map_size: u32,

    #[br(count = submesh_bone_map_size / 2)]
    submesh_bone_map: Vec<u16>,

    #[br(count = header.patch_72_table_size * 16)]
    unknown_72_padding: Vec<u8>,

    padding_amount: u8,
    #[br(count = padding_amount)]
    unknown_padding: Vec<u8>,

    bounding_box: BoundingBox,
    model_bounding_box: BoundingBox,
    water_bounding_box: BoundingBox,
    vertical_fog_bounding_box: BoundingBox,

    #[br(count = header.bone_count)]
    bone_bounding_boxes: Vec<BoundingBox>,
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct ElementId {
    element_id: u32,
    parent_bone_name: u32,
    translate: [f32; 3],
    rotate: [f32; 3],
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub normal: [f32; 3],
    pub bitangent: [f32; 4],
    //pub bitangent1: [f32; 4], // TODO: need to figure out what the heck this could be
    pub color: [f32; 4],

    pub bone_weight: [f32; 4],
    pub bone_id: [u8; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            uv0: [0.0; 2],
            uv1: [0.0; 2],
            normal: [0.0; 3],
            bitangent: [0.0; 4],
            color: [0.0; 4],
            bone_weight: [0.0; 4],
            bone_id: [0u8; 4],
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct NewShapeValue {
    pub base_index: u32,
    pub replacing_vertex: Vertex,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct SubMesh {
    submesh_index: usize,
    pub index_count: u32,
    pub index_offset: u32,
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub name: String,
    pub morphed_vertices: Vec<Vertex>,
}

/// Corresponds to a "Mesh" in an LOD
#[derive(Debug, Clone)]
pub struct Part {
    mesh_index: u16,
    pub vertices: Vec<Vertex>,
    /// Indexed by VertexElement::stream
    pub vertex_streams: Vec<Vec<u8>>,
    pub vertex_stream_strides: Vec<usize>,
    pub indices: Vec<u16>,
    pub material_index: u16,
    pub submeshes: Vec<SubMesh>,
    pub shapes: Vec<Shape>,
    pub part_type: PartType,
}

#[derive(Debug, Clone)]
pub enum PartType {
    Normal,
    Water,
}

#[derive(Debug, Clone)]
pub struct Lod {
    pub parts: Vec<Part>,
}

/// Model file, usually with the `.mdl` file extension.
///
/// Contains vertices, indices and anything else to store 3D model information.
#[derive(Debug, Clone)]
pub struct MDL {
    file_header: ModelFileHeader,
    pub model_data: ModelData,

    pub lods: Vec<Lod>,
    pub affected_bone_names: Vec<String>,
    pub material_names: Vec<String>,
}

impl MDL {
    pub fn replace_vertices(
        &mut self,
        lod_index: usize,
        part_index: usize,
        vertices: &[Vertex],
        indices: &[u16],
        submeshes: &[SubMesh],
    ) {
        let part = &mut self.lods[lod_index].parts[part_index];

        part.indices = Vec::from(indices);
        part.vertices = Vec::from(vertices);

        for (i, submesh) in part.submeshes.iter().enumerate() {
            if i < submeshes.len() {
                self.model_data.submeshes[submesh.submesh_index].index_offset =
                    submeshes[i].index_offset;
                self.model_data.submeshes[submesh.submesh_index].index_count =
                    submeshes[i].index_count;
            }
        }

        // Update vertex count in header
        self.model_data.meshes[part.mesh_index as usize].vertex_count = part.vertices.len() as u16;
        self.model_data.meshes[part.mesh_index as usize].index_count = part.indices.len() as u32;

        // Update the bounding box to the new vertices
        for vertex in vertices {
            self.model_data.bounding_box.min[0] =
                self.model_data.bounding_box.min[0].min(vertex.position[0]);
            self.model_data.bounding_box.min[1] =
                self.model_data.bounding_box.min[1].min(vertex.position[1]);
            self.model_data.bounding_box.min[2] =
                self.model_data.bounding_box.min[2].min(vertex.position[2]);

            self.model_data.bounding_box.max[0] =
                self.model_data.bounding_box.max[0].max(vertex.position[0]);
            self.model_data.bounding_box.max[1] =
                self.model_data.bounding_box.max[1].max(vertex.position[1]);
            self.model_data.bounding_box.max[2] =
                self.model_data.bounding_box.max[2].max(vertex.position[2]);
        }

        self.model_data.model_bounding_box = self.model_data.bounding_box;

        self.update_headers();
    }

    pub fn remove_shape_meshes(&mut self) {
        self.model_data.shape_meshes.clear();
        self.model_data.shape_values.clear();

        for lod in 0..3 {
            for shape in &mut self.model_data.shapes {
                shape.shape_mesh_count[lod] = 0;
                shape.shape_mesh_start_index[lod] = 0;
            }
        }

        self.update_headers();
    }

    pub fn add_shape_mesh(
        &mut self,
        lod_index: usize,
        shape_index: usize,
        shape_mesh_index: usize,
        part_index: usize,
        shape_values: &[NewShapeValue],
    ) {
        let part = &mut self.lods[lod_index].parts[part_index];

        // TODO: this is assuming they are added in order
        if shape_mesh_index == 0 {
            self.model_data.shapes[shape_index].shape_mesh_start_index[lod_index] =
                self.model_data.shape_meshes.len() as u16;
        }

        self.model_data.shape_meshes.push(ShapeMesh {
            mesh_index_offset: self.model_data.meshes[part.mesh_index as usize].start_index,
            shape_value_count: shape_values.len() as u32,
            shape_value_offset: self.model_data.shape_values.len() as u32,
        });

        for shape_value in shape_values {
            part.vertices.push(shape_value.replacing_vertex);

            self.model_data.shape_values.push(ShapeValue {
                base_indices_index: self.model_data.meshes[part.mesh_index as usize].start_index
                    as u16
                    + shape_value.base_index as u16,
                replacing_vertex_index: self.model_data.meshes[part.mesh_index as usize].start_index
                    as u16
                    + (part.vertices.len() - 1) as u16,
            })
        }

        self.model_data.shapes[shape_index].shape_mesh_count[lod_index] += 1;

        self.update_headers();
    }

    pub(crate) fn update_headers(&mut self) {
        // update values
        for i in 0..self.file_header.lod_count {
            let mut vertex_offset = 0;
            let mut combined_index_count = 0;

            for j in self.model_data.lods[i as usize].mesh_index
                ..self.model_data.lods[i as usize].mesh_index
                    + self.model_data.lods[i as usize].mesh_count
            {
                let mesh = &mut self.model_data.meshes[j as usize];

                if (mesh.submesh_index as usize) < self.model_data.submeshes.len() {
                    mesh.start_index =
                        self.model_data.submeshes[mesh.submesh_index as usize].index_offset;
                } else {
                    mesh.start_index = combined_index_count;
                    combined_index_count += mesh.index_count;
                }

                for i in 0..mesh.vertex_stream_count as usize {
                    mesh.vertex_buffer_offsets[i] = vertex_offset;
                    vertex_offset +=
                        mesh.vertex_count as u32 * mesh.vertex_buffer_strides[i] as u32;
                }
            }
        }

        for lod in &mut self.model_data.lods {
            let mut total_vertex_buffer_size = 0;
            let mut total_index_buffer_size = 0;

            // still slightly off?
            for j in lod.mesh_index..lod.mesh_index + lod.mesh_count {
                let vertex_count = self.model_data.meshes[j as usize].vertex_count;
                let index_count = self.model_data.meshes[j as usize].index_count;

                let mut total_vertex_stride: u32 = 0;
                for i in 0..self.model_data.meshes[j as usize].vertex_stream_count as usize {
                    total_vertex_stride +=
                        self.model_data.meshes[j as usize].vertex_buffer_strides[i] as u32;
                }

                total_vertex_buffer_size += vertex_count as u32 * total_vertex_stride;
                total_index_buffer_size += index_count * size_of::<u16>() as u32;
            }

            // TODO: this can definitely be written better
            let mut index_padding = total_index_buffer_size % 16;
            if index_padding == 0 {
                index_padding = 16;
            } else {
                index_padding = 16 - index_padding;
            }

            lod.vertex_buffer_size = total_vertex_buffer_size;
            lod.index_buffer_size = total_index_buffer_size.wrapping_add(index_padding);
        }

        // update lod values
        self.file_header.stack_size = self.file_header.calculate_stack_size();
        self.file_header.runtime_size = self.model_data.calculate_runtime_size();

        let data_offset = self.file_header.runtime_size
            + size_of::<ModelFileHeader>() as u32
            + self.file_header.stack_size;

        let mut overall_offset: u32 = 0;

        for lod in &mut self.model_data.lods {
            // vertex
            lod.vertex_data_offset = data_offset + overall_offset;
            overall_offset += lod.vertex_buffer_size;

            // index
            lod.index_data_offset = data_offset + overall_offset;
            overall_offset += lod.index_buffer_size;

            // edge, but unused?
            //lod.edge_geometry_data_offset = data_offset + overall_offset;
            //overall_offset += lod.edge_geometry_size;

            lod.edge_geometry_data_offset = lod.index_data_offset;
        }

        for i in 0..self.lods.len() {
            self.file_header.vertex_buffer_size[i] = self.model_data.lods[i].vertex_buffer_size;
        }

        for i in 0..self.lods.len() {
            self.file_header.vertex_offsets[i] = self.model_data.lods[i].vertex_data_offset;
        }

        for i in 0..self.lods.len() {
            self.file_header.index_buffer_size[i] = self.model_data.lods[i].index_buffer_size;
        }

        for i in 0..self.lods.len() {
            self.file_header.index_offsets[i] = self.model_data.lods[i].index_data_offset;
        }

        self.model_data.header.shape_count = self.model_data.shapes.len() as u16;
        self.model_data.header.shape_mesh_count = self.model_data.shape_meshes.len() as u16;
        self.model_data.header.shape_value_count = self.model_data.shape_values.len() as u16;
    }
}

impl ReadableFile for MDL {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let endianness = platform.endianness();
        let model_file_header = ModelFileHeader::read_options(&mut cursor, endianness, ()).ok()?;

        let model = ModelData::read_options(&mut cursor, endianness, (&model_file_header,)).ok()?;

        let mut affected_bone_names = vec![];

        for offset in &model.bone_name_offsets {
            let mut offset = *offset;
            let mut string = String::new();

            let mut next_char = model.header.strings[offset as usize] as char;
            while next_char != '\0' {
                string.push(next_char);
                offset += 1;
                next_char = model.header.strings[offset as usize] as char;
            }

            affected_bone_names.push(string);
        }

        let mut material_names = vec![];

        for offset in &model.material_name_offsets {
            let mut offset = *offset;
            let mut string = String::new();

            let mut next_char = model.header.strings[offset as usize] as char;
            while next_char != '\0' {
                string.push(next_char);
                offset += 1;
                next_char = model.header.strings[offset as usize] as char;
            }

            material_names.push(string);
        }

        let mut lods = vec![];

        for i in 0..model.header.lod_count {
            let mut parts = vec![];

            for j in 0..model.meshes.len() as u16 {
                let part_type = get_part_type(&model.lods[i as usize], j);
                if part_type.is_none() {
                    // this mesh isn't in this lod
                    continue;
                }
                let part_type = part_type.unwrap();
                let declaration = &model.header.vertex_declarations[j as usize];
                let vertex_count = model.meshes[j as usize].vertex_count;
                let material_index = model.meshes[j as usize].material_index;

                let mut vertices: Vec<Vertex> = vec![Vertex::default(); vertex_count as usize];

                for k in 0..vertex_count {
                    for element in &declaration.elements {
                        cursor
                            .seek(SeekFrom::Start(
                                (model.lods[i as usize].vertex_data_offset
                                    + model.meshes[j as usize].vertex_buffer_offsets
                                        [element.stream as usize]
                                    + element.offset as u32
                                    + model.meshes[j as usize].vertex_buffer_strides
                                        [element.stream as usize]
                                        as u32
                                        * k as u32) as u64,
                            ))
                            .ok()?;

                        match element.vertex_usage {
                            VertexUsage::Position => match element.vertex_type {
                                VertexType::Single4 => {
                                    vertices[k as usize].position.clone_from_slice(
                                        &MDL::read_single4(&mut cursor, endianness).unwrap()[0..3],
                                    );
                                }
                                VertexType::Half4 => {
                                    vertices[k as usize].position.clone_from_slice(
                                        &MDL::read_half4(&mut cursor, endianness).unwrap()[0..3],
                                    );
                                }
                                VertexType::Single3 => {
                                    vertices[k as usize].position =
                                        MDL::read_single3(&mut cursor, endianness).unwrap();
                                }
                                _ => {
                                    panic!(
                                        "Unexpected vertex type for position: {:#?}",
                                        element.vertex_type
                                    );
                                }
                            },
                            VertexUsage::BlendWeights => match element.vertex_type {
                                VertexType::ByteFloat4 => {
                                    vertices[k as usize].bone_weight =
                                        MDL::read_byte_float4(&mut cursor).unwrap();
                                }
                                VertexType::Byte4 => {
                                    vertices[k as usize].bone_weight =
                                        MDL::read_byte_float4(&mut cursor).unwrap();
                                }
                                VertexType::UnsignedShort4 => {
                                    let bytes =
                                        MDL::read_unsigned_short4(&mut cursor, endianness).unwrap();
                                    vertices[k as usize].bone_weight = [
                                        f32::from(bytes[0]),
                                        f32::from(bytes[1]),
                                        f32::from(bytes[2]),
                                        f32::from(bytes[3]),
                                    ];
                                }
                                _ => {
                                    panic!(
                                        "Unexpected vertex type for blendweight: {:#?}",
                                        element.vertex_type
                                    );
                                }
                            },
                            VertexUsage::BlendIndices => match element.vertex_type {
                                VertexType::Byte4 => {
                                    vertices[k as usize].bone_id =
                                        MDL::read_byte4(&mut cursor).unwrap();
                                }
                                VertexType::UnsignedShort4 => {
                                    let shorts =
                                        MDL::read_unsigned_short4(&mut cursor, endianness).unwrap();
                                    vertices[k as usize].bone_id = [
                                        shorts[0] as u8,
                                        shorts[1] as u8,
                                        shorts[2] as u8,
                                        shorts[3] as u8,
                                    ];
                                }
                                _ => {
                                    panic!(
                                        "Unexpected vertex type for blendindice: {:#?}",
                                        element.vertex_type
                                    );
                                }
                            },
                            VertexUsage::Normal => match element.vertex_type {
                                VertexType::Half4 => {
                                    vertices[k as usize].normal.clone_from_slice(
                                        &MDL::read_half4(&mut cursor, endianness).unwrap()[0..3],
                                    );
                                }
                                VertexType::Single3 => {
                                    vertices[k as usize].normal =
                                        MDL::read_single3(&mut cursor, endianness).unwrap();
                                }
                                VertexType::UnkPS3 => {
                                    // TODO: unsure
                                    vertices[k as usize].normal.clone_from_slice(
                                        &MDL::read_byte_float4(&mut cursor).unwrap()[0..3],
                                    );
                                }
                                _ => {
                                    panic!(
                                        "Unexpected vertex type for normal: {:#?}",
                                        element.vertex_type
                                    );
                                }
                            },
                            VertexUsage::UV => match element.vertex_type {
                                VertexType::ByteFloat4 => {
                                    let combined = MDL::read_byte_float4(&mut cursor).unwrap();

                                    vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                    vertices[k as usize].uv1.clone_from_slice(&combined[2..4]);
                                }
                                VertexType::Half4 => {
                                    let combined =
                                        MDL::read_half4(&mut cursor, endianness).unwrap();

                                    vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                    vertices[k as usize].uv1.clone_from_slice(&combined[2..4]);
                                }
                                VertexType::Single4 => {
                                    let combined =
                                        MDL::read_single4(&mut cursor, endianness).unwrap();

                                    vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                    vertices[k as usize].uv1.clone_from_slice(&combined[2..4]);
                                }
                                VertexType::Half2 => {
                                    let combined =
                                        MDL::read_half2(&mut cursor, endianness).unwrap();

                                    vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                }
                                _ => {
                                    panic!(
                                        "Unexpected vertex type for uv: {:#?}",
                                        element.vertex_type
                                    );
                                }
                            },
                            VertexUsage::BiTangent => match element.vertex_type {
                                VertexType::ByteFloat4 => {
                                    vertices[k as usize].bitangent =
                                        MDL::read_tangent(&mut cursor).unwrap();
                                }
                                _ => {
                                    panic!(
                                        "Unexpected vertex type for bitangent: {:#?}",
                                        element.vertex_type
                                    );
                                }
                            },
                            VertexUsage::Tangent => {
                                match element.vertex_type {
                                    // Used for... terrain..?
                                    VertexType::ByteFloat4 => {}
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for tangent: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                }
                            }
                            VertexUsage::Color => match element.vertex_type {
                                VertexType::ByteFloat4 => {
                                    vertices[k as usize].color =
                                        MDL::read_byte_float4(&mut cursor).unwrap();
                                }
                                _ => {
                                    panic!(
                                        "Unexpected vertex type for color: {:#?}",
                                        element.vertex_type
                                    );
                                }
                            },
                        }
                    }
                }

                cursor
                    .seek(SeekFrom::Start(
                        (model_file_header.index_offsets[i as usize]
                            + (model.meshes[j as usize].start_index * size_of::<u16>() as u32))
                            as u64,
                    ))
                    .ok()?;

                let index_count = model.meshes[j as usize].index_count as usize;
                let indices: Vec<u16> = cursor
                    .read_type_args(endianness, VecArgs::builder().count(index_count).finalize())
                    .ok()?;

                let mut submeshes: Vec<SubMesh> =
                    Vec::with_capacity(model.meshes[j as usize].submesh_count as usize);
                for i in 0..model.meshes[j as usize].submesh_count {
                    submeshes.push(SubMesh {
                        submesh_index: model.meshes[j as usize].submesh_index as usize + i as usize,
                        index_count: model.submeshes
                            [model.meshes[j as usize].submesh_index as usize + i as usize]
                            .index_count,
                        index_offset: model.submeshes
                            [model.meshes[j as usize].submesh_index as usize + i as usize]
                            .index_offset,
                    });
                }

                let mut shapes = vec![];

                for shape in &model.shapes {
                    // Adapted from https://github.com/xivdev/Penumbra/blob/master/Penumbra/Import/Models/Export/MeshExporter.cs
                    let affected_shape_mesh: Vec<&ShapeMesh> = model
                        .shape_meshes
                        .iter()
                        .skip(shape.shape_mesh_start_index[i as usize] as usize)
                        .take(shape.shape_mesh_count[i as usize] as usize)
                        .filter(|shape_mesh| {
                            shape_mesh.mesh_index_offset == model.meshes[j as usize].start_index
                        })
                        .collect();

                    let shape_values: Vec<&ShapeValue> = affected_shape_mesh
                        .iter()
                        .flat_map(|shape_mesh| {
                            model
                                .shape_values
                                .iter()
                                .skip(shape_mesh.shape_value_offset as usize)
                                .take(shape_mesh.shape_value_count as usize)
                        })
                        .filter(|shape_value| {
                            shape_value.base_indices_index
                                >= model.meshes[j as usize].start_index as u16
                                && shape_value.base_indices_index
                                    < (model.meshes[j as usize].start_index
                                        + model.meshes[j as usize].index_count)
                                        as u16
                        })
                        .collect();

                    let mut morphed_vertices = vec![Vertex::default(); vertices.len()];

                    if !shape_values.is_empty() {
                        for shape_value in shape_values {
                            let old_vertex =
                                vertices[indices[shape_value.base_indices_index as usize] as usize];
                            let new_vertex = vertices[shape_value.replacing_vertex_index as usize];
                            let vertex = &mut morphed_vertices
                                [indices[shape_value.base_indices_index as usize] as usize];

                            vertex.position[0] = new_vertex.position[0] - old_vertex.position[0];
                            vertex.position[1] = new_vertex.position[1] - old_vertex.position[1];
                            vertex.position[2] = new_vertex.position[2] - old_vertex.position[2];
                        }

                        let mut offset = shape.string_offset;
                        let mut string = String::new();

                        let mut next_char = model.header.strings[offset as usize] as char;
                        while next_char != '\0' {
                            string.push(next_char);
                            offset += 1;
                            next_char = model.header.strings[offset as usize] as char;
                        }

                        shapes.push(Shape {
                            name: string,
                            morphed_vertices,
                        });
                    }
                }

                let mut vertex_streams = vec![];
                let mut vertex_stream_strides = vec![];
                let mesh = &model.meshes[j as usize];
                for stream in 0..mesh.vertex_stream_count {
                    if stream >= 3 {
                        // TODO: extra strides aren't supported yet!
                        continue;
                    }

                    let mut vertex_data = vec![];
                    let stride = mesh.vertex_buffer_strides[stream as usize];
                    for z in 0..mesh.vertex_count {
                        // TODO: read the entire vertex data into a buffer
                        // Handle the offsets within Novus itself
                        cursor
                            .seek(SeekFrom::Start(
                                (model.lods[i as usize].vertex_data_offset
                                    + model.meshes[j as usize].vertex_buffer_offsets
                                        [stream as usize]
                                    + (z as u32 * stride as u32))
                                    as u64,
                            ))
                            .ok()?;

                        for _ in 0..stride {
                            vertex_data.push(cursor.read_le::<u8>().ok()?);
                        }
                    }

                    vertex_streams.push(vertex_data);
                    vertex_stream_strides
                        .push(mesh.vertex_buffer_strides[stream as usize] as usize);
                }

                parts.push(Part {
                    mesh_index: j,
                    part_type,
                    vertices,
                    indices,
                    material_index,
                    submeshes,
                    shapes,
                    vertex_streams,
                    vertex_stream_strides,
                });
            }

            lods.push(Lod { parts });
        }

        Some(MDL {
            file_header: model_file_header,
            model_data: model,
            lods,
            affected_bone_names,
            material_names,
        })
    }
}

fn get_part_type(lod: &MeshLod, index: u16) -> Option<PartType> {
    if (lod.mesh_index..lod.mesh_index + lod.mesh_count).contains(&index) {
        return Some(PartType::Mesh);
    }

    if (lod.water_mesh_index..lod.water_mesh_index + lod.water_mesh_count).contains(&index) {
        return Some(PartType::WaterMesh);
    }

    None
}

impl WritableFile for MDL {
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();
        let endianness = platform.endianness();

        {
            let mut cursor = Cursor::new(&mut buffer);

            // write file header
            self.file_header
                .write_options(&mut cursor, endianness, ())
                .ok()?;

            self.model_data
                .write_options(&mut cursor, endianness, (&self.file_header,))
                .ok()?;

            for (l, lod) in self.lods.iter().enumerate() {
                for part in lod.parts.iter() {
                    let declaration =
                        &self.model_data.header.vertex_declarations[part.mesh_index as usize];

                    for (k, vert) in part.vertices.iter().enumerate() {
                        for element in &declaration.elements {
                            cursor
                                .seek(SeekFrom::Start(
                                    (self.model_data.lods[l].vertex_data_offset
                                        + self.model_data.meshes[part.mesh_index as usize]
                                            .vertex_buffer_offsets
                                            [element.stream as usize]
                                        + element.offset as u32
                                        + self.model_data.meshes[part.mesh_index as usize]
                                            .vertex_buffer_strides
                                            [element.stream as usize]
                                            as u32
                                            * k as u32) as u64,
                                ))
                                .ok()?;

                            match element.vertex_usage {
                                VertexUsage::Position => match element.vertex_type {
                                    VertexType::Single4 => {
                                        MDL::write_single4(
                                            &mut cursor,
                                            endianness,
                                            &MDL::pad_slice(&vert.position, 1.0),
                                        )
                                        .ok()?;
                                    }
                                    VertexType::Half4 => {
                                        MDL::write_half4(
                                            &mut cursor,
                                            endianness,
                                            &MDL::pad_slice(&vert.position, 1.0),
                                        )
                                        .ok()?;
                                    }
                                    VertexType::Single3 => {
                                        MDL::write_single3(&mut cursor, endianness, &vert.position)
                                            .ok()?;
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for position: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                },
                                VertexUsage::BlendWeights => match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        MDL::write_byte_float4(&mut cursor, &vert.bone_weight)
                                            .ok()?;
                                    }
                                    VertexType::Byte4 => {
                                        MDL::write_byte_float42(&mut cursor, &vert.bone_weight)
                                            .ok()?; // TODO: WRONG!
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for blendweight: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                },
                                VertexUsage::BlendIndices => match element.vertex_type {
                                    VertexType::Byte4 => {
                                        MDL::write_byte4(&mut cursor, &vert.bone_id).ok()?;
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for blendindice: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                },
                                VertexUsage::Normal => match element.vertex_type {
                                    VertexType::Half4 => {
                                        MDL::write_half4(
                                            &mut cursor,
                                            endianness,
                                            &MDL::pad_slice(&vert.normal, 0.0),
                                        )
                                        .ok()?;
                                    }
                                    VertexType::Single3 => {
                                        MDL::write_single3(&mut cursor, endianness, &vert.normal)
                                            .ok()?;
                                    }
                                    VertexType::UnkPS3 => {
                                        // TODO: unsure
                                        MDL::write_byte_float4(
                                            &mut cursor,
                                            &MDL::pad_slice(&vert.normal, 0.0),
                                        )
                                        .ok()?;
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for normal: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                },
                                VertexUsage::UV => match element.vertex_type {
                                    VertexType::Half2 => {
                                        MDL::write_half2(&mut cursor, endianness, &vert.uv0)
                                            .ok()?;
                                    }
                                    VertexType::Half4 => {
                                        let combined =
                                            [vert.uv0[0], vert.uv0[1], vert.uv1[0], vert.uv1[1]];

                                        MDL::write_half4(&mut cursor, endianness, &combined)
                                            .ok()?;
                                    }
                                    VertexType::Single4 => {
                                        let combined =
                                            [vert.uv0[0], vert.uv0[1], vert.uv1[0], vert.uv1[1]];

                                        MDL::write_single4(&mut cursor, endianness, &combined)
                                            .ok()?;
                                    }
                                    VertexType::ByteFloat4 => {
                                        let combined =
                                            [vert.uv0[0], vert.uv0[1], vert.uv1[0], vert.uv1[1]];

                                        MDL::write_tangent(&mut cursor, &combined).ok()?;
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for uv: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                },
                                VertexUsage::BiTangent => match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        MDL::write_tangent(&mut cursor, &vert.bitangent).ok()?;
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for bitangent: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                },
                                VertexUsage::Tangent => {
                                    #[allow(clippy::match_single_binding)] // TODO
                                    match element.vertex_type {
                                        VertexType::ByteFloat4 => {
                                            // TODO: restore
                                            //MDL::write_tangent(&mut cursor, &vert.binormal).ok()?;
                                            MDL::write_tangent(&mut cursor, &[0.0, 0.0, 0.0, 0.0])
                                                .ok()?;
                                        }
                                        _ => {
                                            panic!(
                                                "Unexpected vertex type for tangent: {:#?}",
                                                element.vertex_type
                                            );
                                        }
                                    }
                                }
                                VertexUsage::Color => match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        MDL::write_byte_float4(&mut cursor, &vert.color).ok()?;
                                    }
                                    _ => {
                                        panic!(
                                            "Unexpected vertex type for color: {:#?}",
                                            element.vertex_type
                                        );
                                    }
                                },
                            }
                        }
                    }

                    cursor
                        .seek(SeekFrom::Start(
                            (self.file_header.index_offsets[l]
                                + (self.model_data.meshes[part.mesh_index as usize].start_index
                                    * size_of::<u16>() as u32)) as u64,
                        ))
                        .ok()?;

                    cursor.write_le(&part.indices).ok()?;
                }
            }
        }

        Some(buffer)
    }
}

impl ModelFileHeader {
    pub fn calculate_stack_size(&self) -> u32 {
        // From https://github.com/Ottermandias/Penumbra.GameData/blob/44021b93e6901c84b739bbf4d1c6350f4486cdbf/Files/MdlFile.cs#L11
        self.vertex_declaration_count as u32 * NUM_VERTICES * VERTEX_ELEMENT_SIZE as u32
    }
}

// TODO: From Xande, need to be cleaned up :)
impl ModelData {
    pub fn calculate_runtime_size(&self) -> u32 {
        2   //StringCount
        + 2 // Unknown
        + 4 //StringSize
        + self.header.string_size
        + 56 //ModelHeader
        + (self.element_ids.len() as u32 * 32)
        + (3 * 60) // 3 Lods
        //+ ( /*file.ModelHeader.ExtraLodEnabled ? 40*/ 0 )
        + self.meshes.len() as u32 * 36
        + self.attribute_name_offsets.len() as u32 * size_of::<u32>() as u32
        + self.header.terrain_shadow_mesh_count as u32 * 20
        + self.header.submesh_count as u32 * 16
        + self.header.terrain_shadow_submesh_count as u32 * 10
        + self.material_name_offsets.len() as u32 * size_of::<u32>() as u32
        + self.bone_name_offsets.len() as u32 * size_of::<u32>() as u32
        + self.bone_tables.len() as u32 * 132
        + self.header.shape_count as u32 * 16
        + self.header.shape_mesh_count as u32 * 12
        + self.header.shape_value_count as u32 * 4
        + 4 // SubmeshBoneMapSize
        + self.submesh_bone_map.len() as u32 * 2
        + self.padding_amount as u32 + 1          // PaddingAmount and Padding
        + (4 * 32) // 4 BoundingBoxes
        + (self.header.bone_count as u32 * 32)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::mem::size_of;
    use std::path::PathBuf;

    use vertex_declarations::VERTEX_ELEMENT_SIZE;

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<MDL>();
    }

    #[test]
    fn test_file_header_size() {
        assert_eq!(0x44, size_of::<ModelFileHeader>());
    }

    #[test]
    fn test_vertex_element_size() {
        assert_eq!(8, VERTEX_ELEMENT_SIZE);
    }

    #[test]
    fn test_stack_size() {
        let example_header = ModelFileHeader {
            version: 0,
            stack_size: 0,
            runtime_size: 0,
            vertex_declaration_count: 6,
            material_count: 0,
            vertex_offsets: [0; 3],
            index_offsets: [0; 3],
            vertex_buffer_size: [0; 3],
            index_buffer_size: [0; 3],
            lod_count: 0,
            index_buffer_streaming_enabled: false,
            has_edge_geometry: false,
        };

        assert_eq!(816, example_header.calculate_stack_size());

        let example_header2 = ModelFileHeader {
            version: 0,
            stack_size: 0,
            runtime_size: 0,
            vertex_declaration_count: 2,
            material_count: 0,
            vertex_offsets: [0; 3],
            index_offsets: [0; 3],
            vertex_buffer_size: [0; 3],
            index_buffer_size: [0; 3],
            lod_count: 0,
            index_buffer_streaming_enabled: false,
            has_edge_geometry: false,
        };

        assert_eq!(272, example_header2.calculate_stack_size());
    }

    #[test]
    fn test_update_headers() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("c0201e0038_top_zeroed.mdl");

        let mut mdl = MDL::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        let old_mdl = mdl.clone();

        mdl.update_headers();

        // There should be no changes
        assert_eq!(mdl.file_header, old_mdl.file_header);
        assert_eq!(mdl.model_data, old_mdl.model_data);
    }

    #[test]
    fn test_update_vertices() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("c0201e0038_top_zeroed.mdl");

        let mut mdl = MDL::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        let old_mdl = mdl.clone();

        for l in 0..old_mdl.lods.len() {
            for p in 0..old_mdl.lods[l].parts.len() {
                mdl.replace_vertices(
                    l,
                    p,
                    &old_mdl.lods[l].parts[p].vertices,
                    &old_mdl.lods[l].parts[p].indices,
                    &old_mdl.lods[l].parts[p].submeshes,
                );
            }
        }

        // There should be no changes
        assert_eq!(mdl.file_header, old_mdl.file_header);

        // TODO: bounding box changes this currently :-(
        //assert_eq!(mdl.model_data, old_mdl.model_data);
    }

    #[test]
    fn test_parsing() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("c0201e0038_top_zeroed.mdl");

        let mdl = MDL::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();

        // file header
        assert_eq!(mdl.file_header.version, 16777221);
        assert_eq!(mdl.file_header.stack_size, 816);
        assert_eq!(
            mdl.file_header.stack_size,
            mdl.file_header.calculate_stack_size()
        );
        assert_eq!(mdl.file_header.runtime_size, 12544);
        assert_eq!(
            mdl.file_header.runtime_size,
            mdl.model_data.calculate_runtime_size()
        );
        assert_eq!(mdl.file_header.vertex_declaration_count, 6);
        assert_eq!(mdl.file_header.material_count, 2);
        assert_eq!(mdl.file_header.lod_count, 3);
        assert!(!mdl.file_header.index_buffer_streaming_enabled);
        assert!(!mdl.file_header.has_edge_geometry);

        // model header
        assert_eq!(mdl.model_data.header.radius, 1.5340779);
    }
}
