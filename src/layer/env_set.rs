// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common_file_operations::write_bool_as;

use super::read_bool_from;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum EnvSetShape {
    Ellipsoid = 0x1,
    Cuboid = 0x2,
    Cylinder = 0x3,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(little)]
pub struct EnvSetInstanceObject {
    pub asset_path_offset: u32,
    pub bound_instance_id: u32,
    pub shape: EnvSetShape,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_env_map_shooting_point: bool,
    #[brw(pad_after = 2)] // padding
    pub priority: u8,
    pub effective_range: f32,
    pub interpolation_time: i32,
    pub reverb: f32,
    pub filter: f32,
    pub sound_asset_path_offset: u32,
}
