// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common_file_operations::write_bool_as;

use super::{HeapString, StringHeap, read_bool_from};

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum ModelCollisionType {
    None = 0x0,
    Replace = 0x1,
    Box = 0x2,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct BGInstanceObject {
    #[brw(args(string_heap))]
    pub asset_path: HeapString,
    #[brw(args(string_heap))]
    pub collision_asset_path: HeapString,
    pub collision_type: ModelCollisionType,
    pub attribute_mask: u32,
    pub attribute: u32,
    pub collision_config: i32,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_visible: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub render_shadow_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub render_light_shadow_enabeld: bool,
    pub unk1_padding: u8, // padding
    pub render_model_clip_range: f32,
    pub padding: [u8; 24], // TODO: UNKNOWN, MAYBE WRONG
}
