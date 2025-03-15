use binrw::{binread, binrw};

use super::read_bool_from;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum EnvSetShape {
    Ellipsoid = 0x1,
    Cuboid = 0x2,
    Cylinder = 0x3,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct EnvSetInstanceObject {
    pub asset_path_offset: u32,
    pub bound_instance_id: u32,
    pub shape: EnvSetShape,
    #[br(map = read_bool_from::<u8>)]
    pub is_env_map_shooting_point: bool,
    pub priority: u8,
    padding: u16,
    pub effective_range: f32,
    pub interpolation_time: i32,
    pub reverb: f32,
    pub filter: f32,
    pub sound_asset_path_offset: u32,
}
