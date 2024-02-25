// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};
use std::mem::size_of;

use binrw::{binrw, BinWrite, BinWriterExt};
use binrw::BinRead;
use binrw::BinReaderExt;
use crate::{ByteBuffer, ByteSpan};
use crate::model_vertex_declarations::{vertex_element_parser, VERTEX_ELEMENT_SIZE, vertex_element_writer, VertexDeclaration, VertexElement, VertexType, VertexUsage};

pub const NUM_VERTICES: u32 = 17;

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[brw(little)]
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

    #[br(map = | x: u8 | x != 0)]
    #[bw(map = | x: & bool | -> u8 { if * x { 1 } else { 0 } })]
    pub index_buffer_streaming_enabled: bool,
    #[br(map = | x: u8 | x != 0)]
    #[bw(map = | x: & bool | -> u8 { if * x { 1 } else { 0 } })]
    #[brw(pad_after = 1)]
    pub has_edge_geometry: bool,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Clone,  PartialEq)]
enum ModelFlags1 {
    DustOcclusionEnabled = 0x80,
    SnowOcclusionEnabled = 0x40,
    RainOcclusionEnabled = 0x20,
    Unknown1 = 0x10,
    LightingReflectionEnabled = 0x08,
    WavingAnimationDisabled = 0x04,
    LightShadowDisabled = 0x02,
    ShadowDisabled = 0x01,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Clone, PartialEq)]
enum ModelFlags2 {
    None = 0x0,
    Unknown2 = 0x80,
    BgUvScrollEnabled = 0x40,
    EnableForceNonResident = 0x20,
    ExtraLodEnabled = 0x10,
    ShadowMaskEnabled = 0x08,
    ForceLodRangeEnabled = 0x04,
    EdgeGeometryEnabled = 0x02,
    Unknown3 = 0x01,
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

    unknown4: u16,

    terrain_shadow_submesh_count: u16,

    unknown5: u8,

    bg_change_material_index: u8,
    bg_crest_change_material_index: u8,

    unknown6: u8,
    unknown7: u16,
    unknown8: u16,
    #[brw(pad_after = 6)]
    unknown9: u16
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

    terrain_shadow_mesh_count: u16,
    terrain_shadow_mesh_index: u16,

    vertical_fog_mesh_index: u16,
    vertical_fog_mesh_count: u16,

    // unused on win32 according to lumina devs
    edge_geometry_size: u32,
    edge_geometry_data_offset: u32,

    #[brw(pad_after = 4)]
    polygon_count: u32,

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

#[binrw]
#[derive(Debug, Clone, PartialEq)]
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
    padding: u8
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct TerrainShadowSubmesh {
    index_offset: u32,
    index_count: u32,
    unknown1: u16,
    unknown2: u16
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct Shape {
    string_offset: u32,
    shape_mesh_start_index: [u16; 3],
    shape_mesh_count: [u16; 3]
}

#[binrw]
#[derive(Debug, Clone,  PartialEq)]
#[allow(dead_code)]
struct ShapeMesh {
    mesh_index_offset: u32,
    shape_value_count: u32,
    shape_value_offset: u32
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
struct ShapeValue {
    base_indices_index: u16,
    replacing_vertex_index: u16
}

#[binrw]
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
#[br(import {file_header: &ModelFileHeader})]
#[brw(little)]
struct ModelData {
    #[br(args { vertex_declaration_count: file_header.vertex_declaration_count })]
    header: ModelHeader,

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
    bone_tables: Vec<BoneTable>,

    #[br(count = header.shape_count)]
    shapes: Vec<Shape>,

    #[br(count = header.shape_mesh_count)]
    shape_meshes: Vec<ShapeMesh>,

    #[br(count = header.shape_value_count)]
    shape_values: Vec<ShapeValue>,

    submesh_bone_map_size: u32,

    #[br(count = submesh_bone_map_size / 2)]
    submesh_bone_map: Vec<u16>,

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
#[derive(Debug, Clone,  PartialEq)]
#[allow(dead_code)]
struct ElementId {
    element_id: u32,
    parent_bone_name: u32,
    translate: [f32; 3],
    rotate: [f32; 3],
}

#[derive(Clone, Copy,  PartialEq)]
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

#[derive(Clone, Copy)]
#[repr(C)]
pub struct SubMesh {
    submesh_index: usize,
    pub index_count: u32,
    pub index_offset: u32
}

#[derive(Clone)]
pub struct Part {
    mesh_index: u16,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub material_index: u16,
    pub submeshes: Vec<SubMesh>
}

#[derive(Clone)]
pub struct Lod {
    pub parts: Vec<Part>,
}

#[derive(Clone)]
pub struct MDL {
    file_header: ModelFileHeader,
    model_data: ModelData,

    pub lods: Vec<Lod>,
    pub affected_bone_names: Vec<String>,
    pub material_names: Vec<String>
}

impl MDL {
    pub fn from_existing(buffer: ByteSpan) -> Option<MDL> {
        let mut cursor = Cursor::new(buffer);
        let model_file_header = ModelFileHeader::read(&mut cursor).unwrap();

        let model = ModelData::read_args(&mut cursor, binrw::args! { file_header: &model_file_header }).unwrap();

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

            for j in model.lods[i as usize].mesh_index
                ..model.lods[i as usize].mesh_index + model.lods[i as usize].mesh_count
            {
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
                            VertexUsage::Position => {
                                match element.vertex_type {
                                    VertexType::Single4 => {
                                        vertices[k as usize].position.clone_from_slice(&MDL::read_single4(&mut cursor).unwrap()[0..3]);
                                    }
                                    VertexType::Half4 => {
                                        vertices[k as usize].position.clone_from_slice(&MDL::read_half4(&mut cursor).unwrap()[0..3]);
                                    }
                                    VertexType::Single3 => {
                                        vertices[k as usize].position = MDL::read_single3(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for position: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::BlendWeights => {
                                match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        vertices[k as usize].bone_weight = MDL::read_byte_float4(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for blendweight: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::BlendIndices => {
                                match element.vertex_type {
                                    VertexType::Byte4 => {
                                        vertices[k as usize].bone_id = MDL::read_byte4(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for blendindice: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::Normal => {
                                match element.vertex_type {
                                    VertexType::Half4 => {
                                        vertices[k as usize].normal.clone_from_slice(&MDL::read_half4(&mut cursor).unwrap()[0..3]);
                                    }
                                    VertexType::Single3 => {
                                        vertices[k as usize].normal = MDL::read_single3(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for normal: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::UV => {
                                match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        let combined = MDL::read_byte_float4(&mut cursor).unwrap();

                                        vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                        vertices[k as usize].uv1.clone_from_slice(&combined[2..4]);
                                    }
                                    VertexType::Half4 => {
                                        let combined = MDL::read_half4(&mut cursor).unwrap();

                                        vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                        vertices[k as usize].uv1.clone_from_slice(&combined[2..4]);
                                    }
                                    VertexType::Single4 => {
                                        let combined = MDL::read_single4(&mut cursor).unwrap();

                                        vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                        vertices[k as usize].uv1.clone_from_slice(&combined[2..4]);
                                    }
                                    VertexType::Half2 => {
                                        let combined = MDL::read_half2(&mut cursor).unwrap();

                                        vertices[k as usize].uv0.clone_from_slice(&combined[0..2]);
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for uv: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::BiTangent => {
                                match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        vertices[k as usize].bitangent = MDL::read_tangent(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for bitangent: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::Tangent => {
                                match element.vertex_type {
                                    // Used for... terrain..?
                                    VertexType::ByteFloat4 => {}
                                    _ => {
                                        panic!("Unexpected vertex type for tangent: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::Color => {
                                match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        vertices[k as usize].color = MDL::read_byte_float4(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for color: {:#?}", element.vertex_type);
                                    }
                                }
                            }
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

                // TODO: optimize!
                let mut indices: Vec<u16> =
                    Vec::with_capacity(model.meshes[j as usize].index_count as usize);
                for _ in 0..model.meshes[j as usize].index_count {
                    indices.push(cursor.read_le::<u16>().ok()?);
                }

                let mut submeshes: Vec<SubMesh> = Vec::with_capacity(model.meshes[j as usize].submesh_count as usize);
                for i in 0..model.meshes[j as usize].submesh_count {
                    submeshes.push(SubMesh {
                        submesh_index: model.meshes[j as usize].submesh_index as usize + i as usize,
                        index_count: model.submeshes[model.meshes[j as usize].submesh_index as usize + i as usize].index_count,
                        index_offset: model.submeshes[model.meshes[j as usize].submesh_index as usize + i as usize].index_offset,
                    });
                }

                parts.push(Part { mesh_index: j, vertices, indices, material_index, submeshes });
            }

            lods.push(Lod { parts });
        }

        Some(MDL {
            file_header: model_file_header,
            model_data: model,
            lods,
            affected_bone_names,
            material_names
        })
    }

    pub fn replace_vertices(&mut self, lod_index: usize, part_index: usize, vertices: &[Vertex], indices: &[u16], submeshes: &[SubMesh]) {
        let part = &mut self.lods[lod_index].parts[part_index];

        part.vertices = Vec::from(vertices);
        part.indices = Vec::from(indices);

        for (i, submesh) in part.submeshes.iter().enumerate() {
            if i < submeshes.len() {
                self.model_data.submeshes[submesh.submesh_index].index_offset = submeshes[i].index_offset;
                self.model_data.submeshes[submesh.submesh_index].index_count = submeshes[i].index_count;
            }
        }

        // Update vertex count in header
        self.model_data.meshes[part.mesh_index as usize].vertex_count = part.vertices.len() as u16;
        self.model_data.meshes[part.mesh_index as usize].index_count = part.indices.len() as u32;

        self.update_headers();
    }

    pub(crate) fn update_headers(&mut self) {
        // update values
        for i in 0..self.file_header.lod_count {
            let mut vertex_offset = 0;

            for j in self.model_data.lods[i as usize].mesh_index
                ..self.model_data.lods[i as usize].mesh_index + self.model_data.lods[i as usize].mesh_count
            {
                let mesh = &mut self.model_data.meshes[j as usize];

                mesh.start_index = self.model_data.submeshes[mesh.submesh_index as usize].index_offset;

                for i in 0..mesh.vertex_stream_count as usize {
                    mesh.vertex_buffer_offsets[i] = vertex_offset;
                    vertex_offset += mesh.vertex_count as u32 * mesh.vertex_buffer_strides[i] as u32;
                }
            }
        }

        for lod in &mut self.model_data.lods {
            let mut total_vertex_buffer_size = 0;
            let mut total_index_buffer_size = 0;

            // still slightly off?
            for j in lod.mesh_index
                ..lod.mesh_index + lod.mesh_count
            {
                let vertex_count = self.model_data.meshes[j as usize].vertex_count;
                let index_count = self.model_data.meshes[j as usize].index_count;

                let mut total_vertex_stride: u32 = 0;
                for i in 0..self.model_data.meshes[j as usize].vertex_stream_count as usize {
                    total_vertex_stride += self.model_data.meshes[j as usize].vertex_buffer_strides[i] as u32;
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

        let mut vertex_offset = self.file_header.runtime_size
            + size_of::<ModelFileHeader>() as u32
            + self.file_header.stack_size;

        for lod in &mut self.model_data.lods {
            lod.vertex_data_offset = vertex_offset;

            vertex_offset = lod.vertex_data_offset + lod.vertex_buffer_size;

            lod.index_data_offset = vertex_offset;

            // dummy
            lod.edge_geometry_data_offset = vertex_offset;

            vertex_offset = lod.index_data_offset + lod.index_buffer_size;
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
    }

    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);

            // write file header
            self.file_header.write(&mut cursor).ok()?;

            self.model_data.write(&mut cursor).ok()?;

            for (l, lod) in self.lods.iter().enumerate() {
                for part in lod.parts.iter() {
                    let declaration = &self.model_data.header.vertex_declarations[part.mesh_index as usize];

                    for (k, vert) in part.vertices.iter().enumerate() {
                        for element in &declaration.elements {
                            cursor
                                .seek(SeekFrom::Start(
                                    (self.model_data.lods[l].vertex_data_offset
                                        + self.model_data.meshes[part.mesh_index as usize].vertex_buffer_offsets
                                        [element.stream as usize]
                                        + element.offset as u32
                                        + self.model_data.meshes[part.mesh_index as usize].vertex_buffer_strides
                                        [element.stream as usize]
                                        as u32
                                        * k as u32) as u64,
                                ))
                                .ok()?;

                            match element.vertex_usage {
                                VertexUsage::Position => {
                                    match element.vertex_type {
                                        VertexType::Half4 => {
                                            MDL::write_half4(&mut cursor, &MDL::pad_slice(&vert.position, 1.0)).ok()?;
                                        }
                                        VertexType::Single3 => {
                                            MDL::write_single3(&mut cursor, &vert.position).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for position: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::BlendWeights => {
                                    match element.vertex_type {
                                        VertexType::ByteFloat4 => {
                                            MDL::write_byte_float4(&mut cursor, &vert.bone_weight).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for blendweight: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::BlendIndices => {
                                    match element.vertex_type {
                                        VertexType::Byte4 => {
                                            MDL::write_byte4(&mut cursor, &vert.bone_id).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for blendindice: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::Normal => {
                                    match element.vertex_type {
                                        VertexType::Half4 => {
                                            MDL::write_half4(&mut cursor, &MDL::pad_slice(&vert.normal, 0.0)).ok()?;
                                        }
                                        VertexType::Single3 => {
                                            MDL::write_single3(&mut cursor, &vert.normal).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for normal: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::UV => {
                                    match element.vertex_type {
                                        VertexType::Half4 => {
                                            let combined = [vert.uv0[0], vert.uv0[1], vert.uv1[0], vert.uv1[1]];

                                            MDL::write_half4(&mut cursor, &combined).ok()?;
                                        }
                                        VertexType::Single4 => {
                                            let combined = [vert.uv0[0], vert.uv0[1], vert.uv1[0], vert.uv1[1]];

                                            MDL::write_single4(&mut cursor, &combined).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for uv: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::BiTangent => {
                                    match element.vertex_type {
                                        VertexType::ByteFloat4 => {
                                            MDL::write_tangent(&mut cursor, &vert.bitangent).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for bitangent: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::Tangent => {
                                    match element.vertex_type {
                                        /*VertexType::ByteFloat4 => {
                                            MDL::write_tangent(&mut cursor, &vert.binormal).ok()?;
                                        }*/
                                        _ => {
                                            panic!("Unexpected vertex type for tangent: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::Color => {
                                    match element.vertex_type {
                                        VertexType::ByteFloat4 => {
                                            MDL::write_byte_float4(&mut cursor, &vert.color).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for color: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    cursor
                        .seek(SeekFrom::Start(
                            (self.file_header.index_offsets[l]
                                + (self.model_data.meshes[part.mesh_index as usize].start_index * size_of::<u16>() as u32))
                                as u64,
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
    use std::io::Cursor;
    use std::mem::size_of;
    use std::path::PathBuf;
    use crate::dat::FileType::Model;
    use crate::model::{MDL, ModelFileHeader};
    use crate::model_vertex_declarations::{VERTEX_ELEMENT_SIZE, VertexElement};

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

        let mut mdl = MDL::from_existing(&read(d).unwrap()).unwrap();
        let old_mdl = mdl.clone();

        mdl.update_headers();

        // There should be no changes
        assert_eq!(mdl.file_header, old_mdl.file_header);
        assert_eq!(mdl.model_data, old_mdl.model_data);
    }

    #[test]
    fn test_parsing() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("c0201e0038_top_zeroed.mdl");

        let mut mdl = MDL::from_existing(&read(d).unwrap()).unwrap();

        // file header
        assert_eq!(mdl.file_header.version, 16777221);
        assert_eq!(mdl.file_header.stack_size, 816);
        assert_eq!(mdl.file_header.stack_size, mdl.file_header.calculate_stack_size());
        assert_eq!(mdl.file_header.runtime_size, 12544);
        assert_eq!(mdl.file_header.runtime_size, mdl.model_data.calculate_runtime_size());
        assert_eq!(mdl.file_header.vertex_declaration_count, 6);
        assert_eq!(mdl.file_header.material_count, 2);
        assert_eq!(mdl.file_header.lod_count, 3);
        assert_eq!(mdl.file_header.index_buffer_streaming_enabled, false);
        assert_eq!(mdl.file_header.has_edge_geometry, false);

        // model header
        assert_eq!(mdl.model_data.header.radius, 1.5340779);
    }
}