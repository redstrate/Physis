use std::io::{Cursor, Seek, SeekFrom};
use binrw::binrw;
use crate::gamedata::MemoryBuffer;
use binrw::BinRead;
use binrw::binread;
use half::f16;

#[binrw]
#[derive(Debug)]
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

#[binread]
#[br(repr = u8)]
#[derive(Debug)]
enum ModelFlags1 {
    DustOcclusionEnabled = 0x80,
    SnowOcclusionEnabled = 0x40,
    RainOcclusionEnabled = 0x20,
    Unknown1 = 0x10,
    LightingReflectionEnabled = 0x08,
    WavingAnimationDisabled = 0x04,
    LightShadowDisabled = 0x02,
    ShadowDisabled = 0x01
}

#[binread]
#[br(repr = u8)]
#[derive(Debug)]
enum ModelFlags2 {
    None = 0x0,
    Unknown2 = 0x80,
    BgUvScrollEnabled = 0x40,
    EnableForceNonResident = 0x20,
    ExtraLodEnabled = 0x10,
    ShadowMaskEnabled = 0x08,
    ForceLodRangeEnabled = 0x04,
    EdgeGeometryEnabled = 0x02,
    Unknown3 = 0x01
}

#[binread]
#[derive(Debug)]
pub struct ModelHeader {
    #[br(pad_after = 2)]
    string_count : u16,
    string_size : u32,

    #[br(count = string_size)]
    strings : Vec<u8>,

    radius : f32,

    mesh_count : u16,
    attribute_count : u16,
    submesh_count : u16,
    material_count : u16,
    bone_count : u16,
    bone_table_count : u16,
    shape_count : u16,
    shape_mesh_count : u16,
    shape_value_count : u16,

    lod_count : u8,

    flags1 : ModelFlags1,

    element_id_count : u16,
    terrain_shadow_mesh_count : u8,

    #[br(err_context("radius = {}", radius))]
    flags2 : ModelFlags2,

    model_clip_out_of_distance : f32,
    shadow_clip_out_of_distance : f32,

    #[br(pad_before = 2)]
    #[br(pad_after = 2)]
    terrain_shadow_submesh_count : u16,

    bg_change_material_index : u8,
    #[br(pad_after = 12)]
    bg_crest_change_material_index : u8,
}

#[binread]
#[derive(Debug)]
struct MeshLod {
    mesh_index : u16,
    mesh_count : u16,

    model_lod_range : f32,
    texture_lod_range : f32,

    water_mesh_index : u16,
    water_mesh_count : u16,

    shadow_mesh_index : u16,
    shadow_mesh_count : u16,

    terrain_shadow_mesh_count : u16,
    terrain_shadow_mesh_index : u16,

    vertical_fog_mesh_index : u16,
    vertical_fog_mesh_count : u16,

    // unused on win32 according to lumina devs
    edge_geometry_size : u32,
    edge_geometry_data_offset : u32,

    #[br(pad_after = 4)]
    polygon_count : u32,

    vertex_buffer_size : u32,
    index_buffer_size : u32,
    vertex_data_offset : u32,
    index_data_offset : u32
}

#[binread]
#[derive(Debug)]
struct Mesh {
    #[br(pad_after = 2)]
    vertex_count : u16,
    index_count : u32,

    material_index : u16,
    submesh_index : u16,
    submesh_count : u16,

    bone_table_index : u16,
    start_index : u32,

    vertex_buffer_offsets : [u32; 3],
    vertex_buffer_strides : [u8; 3],

    vertex_stream_count : u8
}

#[binread]
#[derive(Debug)]
struct Submesh {
    index_offset : i32,
    index_count : i32,

    attribute_index_mask : u32,

    bone_start_index : u16,
    bone_count : u16
}

#[binread]
#[derive(Debug)]
struct BoneTable {
    bone_indices : [u16; 64],

    #[br(pad_after = 3)]
    bone_count : u8
}

#[binread]
#[derive(Debug)]
struct BoundingBox {
    min : [f32; 4],
    max : [f32; 4]
}

#[binread]
#[derive(Debug)]
struct ModelData {
    header : ModelHeader,

    #[br(count = header.element_id_count)]
    element_ids : Vec<ElementId>,

    #[br(count = 3)]
    lods : Vec<MeshLod>,

    #[br(count = header.mesh_count)]
    meshes : Vec<Mesh>,

    #[br(count = header.attribute_count)]
    attribute_name_offsets : Vec<u32>,

    // TODO: implement terrain shadow meshes

    #[br(count = header.submesh_count)]
    submeshes : Vec<Submesh>,

    // TODO: implement terrain shadow submeshes

    #[br(count = header.material_count)]
    material_name_offsets : Vec<u32>,

    #[br(count = header.bone_count)]
    bone_name_offsets : Vec<u32>,

    #[br(count = header.bone_table_count)]
    bone_tables : Vec<BoneTable>,

    // TODO: implement shapes

    #[br(temp)]
    submesh_bone_map_size : u32,

    #[br(count = submesh_bone_map_size / 2, err_context("lods = {:#?}", lods))]
    submesh_bone_map : Vec<u16>,

    #[br(temp)]
    padding_amount : u8,

    #[br(pad_before = padding_amount)]
    bounding_box : BoundingBox,
    model_bounding_box : BoundingBox,
    water_bounding_box : BoundingBox,
    vertical_fog_bounding_box : BoundingBox,

    #[br(count = header.bone_count)]
    bone_bounding_boxes : Vec<BoundingBox>
}

#[binread]
#[derive(Debug)]
struct ElementId {
    element_id : u32,
    parent_bone_name : u32,
    translate : [f32; 3],
    rotate : [f32; 3]
}

#[binread]
#[br(repr = u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum VertexType {
    Invalid = 0,
    Single3 = 2,
    Single4 = 3,
    UInt = 5,
    ByteFloat4 = 8,
    Half2 = 13,
    Half4 = 14
}

#[binread]
#[br(repr = u8)]
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

#[binread]
#[derive(Copy, Clone, Debug)]
struct VertexElement {
    stream : u8,
    offset : u8,
    vertex_type : VertexType,
    vertex_usage : VertexUsage,
    #[br(pad_after = 3)]
    usage_index: u8
}

#[derive(Clone)]
#[repr(C)]
pub struct Vertex {
    pub position : [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],

    pub bone_weight: [f32; 4],
    pub bone_id : [u8; 4]
}

pub struct Part {
    pub vertices : Vec<Vertex>,
    pub indices : Vec<u16>
}

pub struct Lod {
    pub parts : Vec<Part>
}

pub struct MDL {
    pub lods : Vec<Lod>
}

impl MDL {
    pub fn from_existing(buffer : &MemoryBuffer) -> Option<MDL> {
        let mut cursor = Cursor::new(buffer);
        let model_file_header = ModelFileHeader::read(&mut cursor).unwrap();

        #[derive(Clone)]
        struct VertexDeclaration {
            elements : Vec<VertexElement>
        }

        let mut vertex_declarations: Vec<VertexDeclaration> = vec![VertexDeclaration{ elements : vec![] }; model_file_header.vertex_declaration_count as usize];
        for declaration in &mut vertex_declarations {
            let mut element = VertexElement::read(&mut cursor).unwrap();

            loop {
                declaration.elements.push(element);

                element = VertexElement::read(&mut cursor).unwrap();

                if element.stream == 255 {
                    break;
                }
            };

            let to_seek = 17 * 8 - (declaration.elements.len() + 1) * 8;
            cursor.seek(SeekFrom::Current(to_seek as i64)).ok()?;
        }

        let model = ModelData::read(&mut cursor).unwrap();

        let mut lods = vec![];

        for i in 0..model.header.lod_count {
            let mut parts = vec![];

            for j in model.lods[i as usize].mesh_index..model.lods[i as usize].mesh_index + model.lods[i as usize].mesh_count {
                let declaration = &vertex_declarations[j as usize];
                let vertex_count = model.meshes[j as usize].vertex_count;

                let default_vertex = Vertex {
                    position: [0.0; 3],
                    uv: [0.0; 2],
                    normal: [0.0; 3],
                    bone_weight: [0.0; 4],
                    bone_id: [0u8; 4]
                };

                let mut vertices: Vec<Vertex> = vec![default_vertex; vertex_count as usize];

                for k in 0..vertex_count {
                    for element in &declaration.elements {
                        cursor.seek(SeekFrom::Start((model.lods[i as usize].vertex_data_offset +
                            model.meshes[j as usize].vertex_buffer_offsets[element.stream as usize] +
                            element.offset as u32 +
                            model.meshes[i as usize].vertex_buffer_strides[element.stream as usize] as u32 * k as u32) as u64)).ok()?;

                        match element.vertex_usage {
                            VertexUsage::Position => {
                                vertices[k as usize].position = <[f32; 3]>::read(&mut cursor).unwrap();
                            }
                            VertexUsage::BlendWeights => {
                                vertices[k as usize].bone_weight = <[f32; 4]>::read(&mut cursor).unwrap();
                            }
                            VertexUsage::BlendIndices => {
                                vertices[k as usize].bone_id = <[u8; 4]>::read(&mut cursor).unwrap();
                            }
                            VertexUsage::Normal => {
                                // TODO: normals are assumed to be half4
                                vertices[k as usize].normal[0] = f16::from_bits(<u16 as BinRead>::read(&mut cursor).unwrap()).to_f32();
                                vertices[k as usize].normal[1] = f16::from_bits(<u16 as BinRead>::read(&mut cursor).unwrap()).to_f32();
                                vertices[k as usize].normal[2] = f16::from_bits(<u16 as BinRead>::read(&mut cursor).unwrap()).to_f32();
                            }
                            VertexUsage::UV => {
                                vertices[k as usize].uv[0] = f16::from_bits(<u16 as BinRead>::read(&mut cursor).unwrap()).to_f32();
                                vertices[k as usize].uv[1] = f16::from_bits(<u16 as BinRead>::read(&mut cursor).unwrap()).to_f32();
                            }
                            VertexUsage::Tangent2 => {}
                            VertexUsage::Tangent1 => {}
                            VertexUsage::Color => {}
                        }
                    }
                }

                cursor.seek(SeekFrom::Start((model_file_header.index_offsets[i as usize] + (model.meshes[j as usize].start_index * 2)) as u64)).ok()?;

                // TODO: optimize!
                let mut indices : Vec<u16> = Vec::with_capacity(model.meshes[j as usize].index_count as usize);
                for _ in 0..model.meshes[j as usize].index_count {
                    indices.push(<u16 as BinRead>::read(&mut cursor).unwrap());
                }

                parts.push(Part {
                    vertices,
                    indices
                });
            }

            lods.push(Lod {
                parts
            });
        }

        Some(MDL {
            lods
        })
    }
}