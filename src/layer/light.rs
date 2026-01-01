// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common_file_operations::write_bool_as;

use super::{ColorHDRI, read_bool_from};

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum LightType {
    None = 0x0,
    Directional = 0x1,
    Point = 0x2,
    Spot = 0x3,
    Plane = 0x4,
    Line = 0x5,
    Specular = 0x6,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum PointLightType {
    Sphere = 0x0,
    Hemisphere = 0x1,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct LightInstanceObject {
    pub light_type: LightType,
    pub attenuation: f32,
    pub range_rate: f32,
    pub point_light_type: PointLightType,
    pub attenuation_cone_coefficient: f32,
    pub cone_degree: f32,
    pub texture_path_offset: u32,
    pub diffuse_color_hdri: ColorHDRI,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 3)] // padding
    pub follows_directional_light: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub specular_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub bg_shadow_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)] // padding
    pub character_shadow_enabled: bool,
    pub shadow_clip_range: f32,
    pub plane_light_rotation_x: f32,
    pub plane_light_rotation_y: f32,
    #[brw(pad_after = 1)] // padding
    pub merge_group_id: u16,
}
