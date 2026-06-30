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
    attribute_mask: u32,
    attribute: u32,
    push_player_out: u8,
    padding: [u8; 3],
    #[brw(args(heap_pointer, string_heap))]
    pub collision_asset_path: HeapString,
    padding2: u32,
}

#[binrw]
#[repr(C)]
#[brw(repr = i32)]
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
    #[brw(pad_after = 5)] // padding
    pub enabled: bool,
}
