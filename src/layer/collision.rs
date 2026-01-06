// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapString, StringHeap},
};

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct CollisionBoxInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    attribute_mask: u32,
    attribute: u32,
    push_player_out: u8,
    padding: [u8; 3],
    // TODO: this seems... wrong
    #[brw(args(string_heap))]
    collision_asset_path: HeapString,
    padding2: u32,
}

#[binrw]
#[repr(C)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TriggerBoxShape {
    None = 0x0,
    Box = 0x1,
    Sphere = 0x2,
    Cylinder = 0x3,
    Plane = 0x4,
    Mesh = 0x5,
    PlaneTwoSided = 0x6,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct TriggerBoxInstanceObject {
    pub trigger_box_shape: TriggerBoxShape,
    pub priority: i16,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 5)] // padding
    pub enabled: bool,
}
