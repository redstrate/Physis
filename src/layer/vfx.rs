// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    Color, ColorIntensity,
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapString, StringHeap},
};

/// Visual effect object.
#[binrw]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
#[derive(Debug, PartialEq, Clone)]
pub struct VfxInstanceObject {
    /// Path to an `.avfx` file.
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
    #[brw(pad_after = 4)] // padding, not read
    /// Distance at which soft particles should begin to show.
    pub soft_particle_fade_range: f32,
    /// Tints the VFX to this color.
    pub color: Color,
    /// Whether the VFX is visible and playing.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub active: bool,
    /// If set, 10000.0 is used in lieu of `fade_far_end` if that is 0.0. Also does something else?
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk1: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)] // padding, not read
    pub unk2: bool,
    /// Distance where the near fade starts. Only works if `fade_near_start` != `fade_near_end` and both values are not zero.
    ///
    /// If `instance_id` is >= 8962180, this is clamped to a maximum of 1000.0.
    pub fade_near_start: f32,
    /// Distance where the near fade stops. Only works if `fade_near_start` != `fade_near_end` and both values are not zero.
    ///
    /// If `instance_id` is >= 8962180, this is clamped to a maximum of 1000.0.
    pub fade_near_end: f32,
    /// Distance where the far fade starts. Only works if `fade_far_start` != `fade_far_end` and both values are not zero.
    ///
    /// If `instance_id` is >= 8962180, this is clamped to a maximum of 1000.0.
    pub fade_far_start: f32,
    /// Distance where the far fade stops. Only works if `fade_far_start` != `fade_far_end` and both values are not zero.
    ///
    /// Not used if `unk1` is true and this value is 0.0.
    pub fade_far_end: f32,
    /// Modifies the Z level this VFX is drawn at.
    pub z_correct: f32,
    /// Doesn't seem to do anything?
    pub unk3: f32,
}

impl Default for VfxInstanceObject {
    fn default() -> Self {
        Self {
            asset_path: Default::default(),
            soft_particle_fade_range: Default::default(),
            color: Default::default(),
            active: true,
            unk1: Default::default(),
            unk2: Default::default(),
            fade_near_start: Default::default(),
            fade_near_end: Default::default(),
            fade_far_start: Default::default(),
            fade_far_end: Default::default(),
            z_correct: Default::default(),
            unk3: 1.0,
        }
    }
}

#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum LineStyle {
    #[default]
    Red = 1,
    Blue = 2,
    RedFar = 3,
}

/// Generic VFX that are those dotted lines used for zone transitions and boundaries.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct LineVFXInstanceObject {
    pub line_style: LineStyle,
}

/// 2D decal drawn on top of a 3D object.
#[binrw]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct DecalInstanceObject {
    /// Path to an `.tex` file.
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
}

/// (Presumably) a volumetric cloud.
///
/// This does not currently function in the Dawntrail client.
#[binrw]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct VolumetricCloudInstanceObject {
    /// Path to a `.cldb` file.
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
    pub color: ColorIntensity,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub active: bool,
    pub unk1: [u8; 3],
    pub unk2: u32,
}
