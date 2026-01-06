// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::write_bool_as,
    layer::{HeapString, StringHeap},
};

use super::read_bool_from;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum DoorState {
    Auto = 0x1,
    Open = 0x2,
    Closed = 0x3,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum RotationState {
    Rounding = 0x1,
    Stopped = 0x2,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum TransformState {
    Play = 0x0,
    Stop = 0x1,
    Replay = 0x2,
    Reset = 0x3,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum ColourState {
    Play = 0x0,
    Stop = 0x1,
    Replay = 0x2,
    Reset = 0x3,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct SharedGroupInstance {
    /// The path to the `.sgb` file.
    #[brw(args(string_heap))]
    pub asset_path: HeapString,
    pub initial_door_state: DoorState,
    pub overriden_members: i32,
    pub overriden_members_count: i32,
    pub initial_rotation_state: RotationState,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub random_timeline_auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub random_timeline_loop_playback: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)] // padding
    pub collision_controllable_without_eobj: bool,
    pub bound_client_path_instance_id: u32,
    // TODO: read move path settings from this offset
    pub move_path_settings: i32,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 3)] // padding
    pub not_create_navimesh_door: bool,
    pub initial_transform_state: TransformState,
    pub initial_color_state: ColourState,
    pub unk1: [u8; 60],
}
