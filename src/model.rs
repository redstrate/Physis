use binrw::binrw;

#[binrw]
pub struct ModelHeader {
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

pub struct Model {}

impl Model {
    pub fn from_existing() {}
}