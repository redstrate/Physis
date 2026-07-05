// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::write_bool_as,
    string_heap::{HeapPointer, HeapString},
};

use super::{StringHeap, read_bool_from};

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ModelCollisionType {
    #[default]
    None = 0x0,
    Replace = 0x1,
    Box = 0x2,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub struct BgPartInstanceObject {
    /// Path to a `.mdl` for the visual model.
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
    /// Path to a `.pcb` for the collision model.
    #[brw(args(heap_pointer, string_heap))]
    pub collision_asset_path: HeapString,
    pub collision_type: ModelCollisionType,
    pub collision_material_mask_low: u32,
    pub collision_material_id_low: u32,
    pub collision_material_mask_high: u32,
    pub collision_material_id_high: u32,
    pub unk_offset: i32, // TODO: probably some sort of collision config
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_visible: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub render_shadow_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)] // padding, not read
    pub render_light_shadow_enabled: bool,
    pub render_model_clip_range: f32,
    pub unk_float: f32,
}
