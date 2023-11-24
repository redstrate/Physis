// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom, Write};

use binrw::{BinResult, binrw, BinWrite, BinWriterExt};
use binrw::BinRead;
use binrw::BinReaderExt;
use half::f16;
use crate::{ByteBuffer, ByteSpan};

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct ModelFileHeader {
    pub(crate) version: u32,

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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModelHeader {
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

    #[brw(pad_before = 2)]
    #[brw(pad_after = 2)]
    terrain_shadow_submesh_count: u16,

    bg_change_material_index: u8,
    #[brw(pad_after = 12)]
    bg_crest_change_material_index: u8,
}

#[binrw]
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct Submesh {
    index_offset: i32,
    index_count: i32,

    attribute_index_mask: u32,

    bone_start_index: u16,
    bone_count: u16,
}

#[binrw]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BoneTable {
    bone_indices: [u16; 64],

    #[brw(pad_after = 3)]
    bone_count: u8,
}

#[binrw]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BoundingBox {
    min: [f32; 4],
    max: [f32; 4],
}

#[binrw]
#[derive(Debug, Clone)]
#[allow(dead_code)]
#[brw(little)]
struct ModelData {
    header: ModelHeader,

    #[br(count = header.element_id_count)]
    element_ids: Vec<ElementId>,

    #[br(count = 3)]
    lods: Vec<MeshLod>,

    #[br(count = header.mesh_count)]
    meshes: Vec<Mesh>,

    #[br(count = header.attribute_count)]
    attribute_name_offsets: Vec<u32>,

    // TODO: implement terrain shadow meshes
    #[br(count = header.submesh_count)]
    submeshes: Vec<Submesh>,

    // TODO: implement terrain shadow submeshes
    #[br(count = header.material_count)]
    material_name_offsets: Vec<u32>,

    #[br(count = header.bone_count)]
    bone_name_offsets: Vec<u32>,

    #[br(count = header.bone_table_count)]
    bone_tables: Vec<BoneTable>,

    // TODO: implement shapes
    submesh_bone_map_size: u32,

    #[br(count = submesh_bone_map_size / 2, err_context("lods = {:#?}", lods))]
    submesh_bone_map: Vec<u16>,

    // TODO: what actually is this?
    padding_amount: u8,

    #[br(pad_before = padding_amount)]
    #[bw(pad_before = *padding_amount)]
    bounding_box: BoundingBox,
    model_bounding_box: BoundingBox,
    water_bounding_box: BoundingBox,
    vertical_fog_bounding_box: BoundingBox,

    #[br(count = header.bone_count)]
    bone_bounding_boxes: Vec<BoundingBox>,
}

#[binrw]
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ElementId {
    element_id: u32,
    parent_bone_name: u32,
    translate: [f32; 3],
    rotate: [f32; 3],
}

#[binrw]
#[brw(repr = u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum VertexType {
    Invalid = 0,
    Single3 = 2,
    Single4 = 3,
    UInt = 5,
    ByteFloat4 = 8,
    Half2 = 13,
    Half4 = 14,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Copy, Clone, Debug)]
enum VertexUsage {
    Position = 0,
    BlendWeights = 1,
    BlendIndices = 2,
    Normal = 3,
    UV = 4,
    Tangent2 = 5,
    Tangent1 = 6,
    Color = 7,
}

#[binrw]
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
#[brw(little)]
struct VertexElement {
    stream: u8,
    offset: u8,
    vertex_type: VertexType,
    vertex_usage: VertexUsage,
    #[brw(pad_after = 3)]
    usage_index: u8,
}

#[derive(Clone)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv0: [f32; 2],
    pub uv1: [f32; 2],
    pub normal: [f32; 3],
    pub tangent1: [u8; 4],
    pub tangent2: [u8; 4],
    pub color: [f32; 4],

    pub bone_weight: [f32; 4],
    pub bone_id: [u8; 4],
}

pub struct Part {
    mesh_index: u16,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub material_index: u16
}

pub struct Lod {
    pub parts: Vec<Part>,
}

#[derive(Clone)]
struct VertexDeclaration {
    elements: Vec<VertexElement>,
}

pub struct MDL {
    file_header: ModelFileHeader,
    vertex_declarations: Vec<VertexDeclaration>,
    model_data: ModelData,

    pub lods: Vec<Lod>,
    pub affected_bone_names: Vec<String>,
    pub material_names: Vec<String>
}

impl MDL {
    pub fn from_existing(buffer: ByteSpan) -> Option<MDL> {
        let mut cursor = Cursor::new(buffer);
        let model_file_header = ModelFileHeader::read(&mut cursor).unwrap();

        let mut vertex_declarations: Vec<VertexDeclaration> =
            vec![
                VertexDeclaration { elements: vec![] };
                model_file_header.vertex_declaration_count as usize
            ];
        for declaration in &mut vertex_declarations {
            let mut element = VertexElement::read(&mut cursor).unwrap();

            loop {
                declaration.elements.push(element);

                element = VertexElement::read(&mut cursor).unwrap();

                if element.stream == 255 {
                    break;
                }
            }

            let to_seek = 17 * 8 - (declaration.elements.len() + 1) * 8;
            cursor.seek(SeekFrom::Current(to_seek as i64)).ok()?;
        }

        let model = ModelData::read(&mut cursor).unwrap();

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
                let declaration = &vertex_declarations[j as usize];
                let vertex_count = model.meshes[j as usize].vertex_count;
                let material_index = model.meshes[j as usize].material_index;

                let default_vertex = Vertex {
                    position: [0.0; 3],
                    uv0: [0.0; 2],
                    uv1: [0.0; 2],
                    normal: [0.0; 3],
                    tangent1: [0u8; 4],
                    tangent2: [0u8; 4],
                    color: [0.0; 4],
                    bone_weight: [0.0; 4],
                    bone_id: [0u8; 4],
                };

                let mut vertices: Vec<Vertex> = vec![default_vertex; vertex_count as usize];

                for k in 0..vertex_count {
                    for element in &declaration.elements {
                        cursor
                            .seek(SeekFrom::Start(
                                (model.lods[i as usize].vertex_data_offset
                                    + model.meshes[j as usize].vertex_buffer_offsets
                                        [element.stream as usize]
                                    + element.offset as u32
                                    + model.meshes[j as usize].vertex_buffer_strides // TODO: is j really correct? this might fix the broken LoDs
                                        [element.stream as usize]
                                        as u32
                                        * k as u32) as u64,
                            ))
                            .ok()?;

                        match element.vertex_usage {
                            VertexUsage::Position => {
                                match element.vertex_type {
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
                                    VertexType::UInt => {
                                        vertices[k as usize].bone_id = MDL::read_uint(&mut cursor).unwrap();
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
                                    _ => {
                                        panic!("Unexpected vertex type for uv: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::Tangent2 => {
                                match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        vertices[k as usize].tangent2 = MDL::read_uint(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for tangent2: {:#?}", element.vertex_type);
                                    }
                                }
                            }
                            VertexUsage::Tangent1 => {
                                match element.vertex_type {
                                    VertexType::ByteFloat4 => {
                                        vertices[k as usize].tangent1 = MDL::read_uint(&mut cursor).unwrap();
                                    }
                                    _ => {
                                        panic!("Unexpected vertex type for tangent1: {:#?}", element.vertex_type);
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
                            + (model.meshes[j as usize].start_index * 2))
                            as u64,
                    ))
                    .ok()?;

                // TODO: optimize!
                let mut indices: Vec<u16> =
                    Vec::with_capacity(model.meshes[j as usize].index_count as usize);
                for _ in 0..model.meshes[j as usize].index_count {
                    indices.push(cursor.read_le::<u16>().ok()?);
                }

                parts.push(Part { mesh_index: j, vertices, indices, material_index });
            }

            lods.push(Lod { parts });
        }

        Some(MDL {
            file_header: model_file_header,
            vertex_declarations,
            model_data: model,
            lods,
            affected_bone_names,
            material_names
        })
    }

    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);

            // write file header
            self.file_header.write(&mut cursor).ok()?;

            // write vertex declarations
            for declaration in &self.vertex_declarations {
                for element in &declaration.elements {
                    element.write(&mut cursor).ok()?;
                }

                cursor.write_all(&[255u8]).ok()?;

                // We have a -1 here like we do in read, because writing the EOF (255) pushes our cursor forward.
                let to_seek = 17 * 8 - (declaration.elements.len()) * 8 - 1;
                cursor.seek(SeekFrom::Current(to_seek as i64)).ok()?;
            }

            self.model_data.write(&mut cursor).ok()?;

            for (l, lod) in self.lods.iter().enumerate() {
                for (i, part) in lod.parts.iter().enumerate() {
                    let declaration = &self.vertex_declarations[part.mesh_index as usize];

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
                                            MDL::write_single4(&mut cursor, &MDL::pad_slice(&vert.position, 1.0)).ok()?;
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
                                        VertexType::UInt => {
                                            MDL::write_uint(&mut cursor, &vert.bone_id).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for blendindice: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::Normal => {
                                    match element.vertex_type {
                                        VertexType::Half4 => {
                                            MDL::write_half4(&mut cursor, &MDL::pad_slice(&vert.normal, 1.0)).ok()?;
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
                                VertexUsage::Tangent2 => {
                                    match element.vertex_type {
                                        VertexType::ByteFloat4 => {
                                            MDL::write_uint(&mut cursor, &vert.tangent2).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for tangent2: {:#?}", element.vertex_type);
                                        }
                                    }
                                }
                                VertexUsage::Tangent1 => {
                                    match element.vertex_type {
                                        VertexType::ByteFloat4 => {
                                            MDL::write_uint(&mut cursor, &vert.tangent1).ok()?;
                                        }
                                        _ => {
                                            panic!("Unexpected vertex type for tangent1: {:#?}", element.vertex_type);
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
                                + (self.model_data.meshes[part.mesh_index as usize].start_index * 2))
                                as u64,
                        ))
                        .ok()?;

                    for indice in &part.indices {
                        cursor.write_le::<u16>(&indice).ok()?;
                    }
                }
            }
        }

        Some(buffer)
    }

    fn read_byte_float4(cursor: &mut Cursor<ByteSpan>) -> Option<[f32; 4]> {
        // TODO: hmmm
        fn round(x: f32) -> f32 {
            (x * 100.0).round() / 100.0
        }

        Some([
            (f32::from(cursor.read_le::<u8>().ok()?) / 255.0),
            (f32::from(cursor.read_le::<u8>().ok()?) / 255.0),
            (f32::from(cursor.read_le::<u8>().ok()?) / 255.0),
            (f32::from(cursor.read_le::<u8>().ok()?) / 255.0)
        ])
    }

    fn write_byte_float4<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 4]) -> BinResult<()> {
        cursor.write_le::<[u8; 4]>(&[
            (vec[0] * 255.0).round() as u8,
            (vec[1] * 255.0).round() as u8,
            (vec[2] * 255.0).round() as u8,
            (vec[3] * 255.0).round() as u8])
    }

    fn read_half4(cursor: &mut Cursor<ByteSpan>) -> Option<[f32; 4]> {
        Some([
             f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32(),
             f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32(),
             f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32(),
             f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32()
        ])
    }

    fn write_half4<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 4]) -> BinResult<()> {
        cursor.write_le::<[u16; 4]>(&[
            f16::from_f32(vec[0]).to_bits(),
            f16::from_f32(vec[1]).to_bits(),
            f16::from_f32(vec[2]).to_bits(),
            f16::from_f32(vec[3]).to_bits()])
    }

    fn read_uint(cursor: &mut Cursor<ByteSpan>) -> BinResult<[u8; 4]> {
        cursor.read_le::<[u8; 4]>()
    }

    fn write_uint<T: BinWriterExt>(cursor: &mut T, vec: &[u8; 4]) -> BinResult<()> {
        cursor.write_le::<[u8; 4]>(vec)
    }

    fn read_single3(cursor: &mut Cursor<ByteSpan>) -> BinResult<[f32; 3]> {
        cursor.read_le::<[f32; 3]>()
    }

    fn write_single3<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 3]) -> BinResult<()> {
        cursor.write_le::<[f32; 3]>(vec)
    }

    fn read_single4(cursor: &mut Cursor<ByteSpan>) -> BinResult<[f32; 4]> {
        cursor.read_le::<[f32; 4]>()
    }

    fn write_single4<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 4]) -> BinResult<()> {
        cursor.write_le::<[f32; 4]>(vec)
    }

    fn pad_slice<const N: usize>(small_slice: &[f32; N], fill: f32) -> [f32; 4] {
        let mut bigger_slice: [f32; 4] = [fill, fill, fill, fill];
        bigger_slice[..N].copy_from_slice(&small_slice[..N]);
        bigger_slice
    }
}


#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::model::MDL;

    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            for i in 0..4 {
                if !($x[i] - $y[i] < $d || $y[i] - $x[i] < $d) { panic!(); }
            }
        }
    }

    #[test]
    fn byte_float4() {
        let a = [0.0, 1.0, 0.5, 0.25];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_byte_float4(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());

        let b = MDL::read_byte_float4(&mut read_cursor).unwrap();
        assert_delta!(b, a, 0.1);
    }

    #[test]
    fn half4() {
        let a = [0.0, 1.0, 0.5, 0.25];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_half4(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_half4(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn uint() {
        let a = [5u8, 0u8, 3u8, 15u8];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_uint(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_uint(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn single3() {
        let a = [3.0, 0.0, -1.0];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_single3(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_single3(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn single4() {
        let a = [3.0, 0.0, -1.0, 12.0];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_single4(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_single4(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn pad_slice() {
        let a = [3.0, 0.0, -1.0];
        let b = [3.0, 0.0, -1.0, 1.0];

        assert_eq!(MDL::pad_slice(&a, 1.0), b);
    }
}
