// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapString, StringHeap},
};

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub struct CollisionBoxInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    material_mask_low: u32,
    material_id_low: u32,
    material_mask_high: u32,
    material_id_high: u32,
    #[brw(pad_after = 3)] // Padding, not read
    layer_mask_is_43h: u8,

    /// Path to the PCB if `trigger_box_shape` is `Mesh`.
    #[brw(args(heap_pointer, string_heap))]
    pub collision_asset_path: HeapString,
}

#[binrw]
#[repr(C)]
#[brw(repr = u32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum TriggerBoxShape {
    #[default]
    None,
    Box,
    Sphere,
    Cylinder,
    Plane,
    Mesh,
    PlaneTwoSided,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TriggerBoxInstanceObject {
    pub trigger_box_shape: TriggerBoxShape,
    pub priority: i16,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 5)] // Padding, not read
    pub enabled: bool,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct CullingBoxInstanceObject {
    unk1: u32,
}
