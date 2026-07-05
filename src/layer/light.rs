// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::write_bool_as,
    string_heap::{HeapPointer, HeapString, StringHeap},
};

use super::{ColorHDRI, read_bool_from};

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(i32)]
pub enum LightShape {
    /// Invalid/empty light.
    None = 0,
    /// A uniform ambient light with no distinct shape that applies everywhere.
    #[default]
    World = 1,
    /// A light that emits from its location equally in all directions.
    Point = 2,
    /// A light that emits from its location in a cone along its positive Z axis.
    Spot = 3,
    /// A light that emits from its location in a parallelogram along its positive Z axis.
    Flat = 4,
    Line = 5,
    Specular = 6,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(i32)]
pub enum PointLightType {
    #[default]
    Sphere = 0x0,
    Hemisphere = 0x1,
}

#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub struct LightInstanceObject {
    /// What type of light this is.
    pub shape: LightShape,
    /// The attenuation factor. I'm unsure how it's used, but usually the higher this is the "softer" and more spread out the light is.
    /// The game will also automagically determine the size of the specular highlight from this and other related values.
    pub attenuation: f32,
    /// Seems to also affect attenuation in some way, almost like a factor.
    pub range: f32,
    pub point_light_type: PointLightType,
    pub attenuation_cone_coefficient: f32,
    pub spot_angle: f32,
    /// Path to a `.tex` file used as a light cookie.
    #[brw(args(heap_pointer, string_heap))]
    pub texture_path: HeapString,
    /// The color and intensity for this light.
    #[brw(pad_after = 4)]
    pub color: ColorHDRI,
    /// Whether specular highlights for this light is enabled.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub enable_specular_highlights: bool,
    /// Whether BG part objects with `cast_light_shadow` will cast shadows in this light.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub enable_bg_part_shadows: bool,
    /// Whether character objects will cast shadows in this light.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub enable_character_shadows: bool,
    /// Is probably another shadow-related flag but I'm unsure of what it controls.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk_bool: bool,
    /// The near plane used while rendering shadows for this light.
    pub shadow_plane_near: f32,
    /// Only set for Flat lights. In radians.
    pub flat_light_skew_angle: [f32; 2],
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk1: f32,
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk2: f32,
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk3: f32,
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk4: f32,
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk5: f32,
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk6: f32,
    /// Must be kept at zero, otherwise the light fails to show up?
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk7: i32,
    /// Only used if `merge_group_id` is 0x56373030.
    pub unk8: f32,
    /// Only used if `merge_group_id` is 0x56373030.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 3)] // padding, not read
    pub unk9: bool,
    /// Usually 0x56373030, unsure the significance of this value.
    pub unk10: i32,
}

impl Default for LightInstanceObject {
    fn default() -> Self {
        Self {
            shape: Default::default(),
            attenuation: 2.0,
            range: 1.0,
            point_light_type: Default::default(),
            attenuation_cone_coefficient: 0.5,
            spot_angle: 45.0,
            texture_path: Default::default(),
            color: Default::default(),
            enable_specular_highlights: Default::default(),
            enable_bg_part_shadows: Default::default(),
            enable_character_shadows: Default::default(),
            unk_bool: Default::default(),
            shadow_plane_near: 0.1,
            flat_light_skew_angle: Default::default(),
            // These seem to be the values for most lights, so I'm assuming they're the default.
            unk1: 1.0,
            unk2: 0.0001,
            unk3: 0.002,
            unk4: 40.0,
            unk5: 400.0,
            unk6: 4.0,
            unk7: 0,
            unk8: 0.0,
            unk9: false,
            unk10: 1446457392,
        }
    }
}
