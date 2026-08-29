// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused)] // Clippy thinks SIZE is unused, but binrw uses it

use std::io::SeekFrom;

use binrw::binrw;

use crate::{
    common_file_operations::write_bool_as,
    layer::StringHeap,
    string_heap::{HeapPointer, HeapString},
};

use super::read_bool_from;

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum DoorState {
    #[default]
    Auto = 1,
    Open = 2,
    Closed = 3,
}

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum RotationState {
    #[default]
    Rounding = 1,
    Stopped = 2,
}

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum TransformState {
    #[default]
    Play = 0,
    Stop = 1,
    Replay = 2,
    Reset = 3,
}

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum ColourState {
    #[default]
    Play = 0,
    Stop = 1,
    Replay = 2,
    Reset = 3,
}

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum MovePathMode {
    #[default]
    None = 0,
    SharedGroupAction = 1,
    Timeline = 2,
}

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum RotationType {
    #[default]
    NoRotate = 0,
    AllAxis = 1,
    YAxisOnly = 2,
}

#[binrw]
#[derive(Debug, PartialEq, Default, Clone)]
pub struct MovePathSettings {
    pub mode: MovePathMode,
    #[brw(pad_after = 1)] // padding, not read
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub auto_play: bool,
    pub time: u16,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub loop_animation: bool,
    #[brw(pad_after = 2)] // padding, not read
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub reverse: bool,
    pub rotation: RotationType,
    pub accelerate_time: u16,
    pub decelerate_time: u16,
    pub vertical_swing_range: [f32; 2],
    pub horizontal_swing_range: [f32; 2],
    pub swing_move_speed_range: [f32; 2],
    pub swing_rotation: [f32; 2],
    pub swing_rotation_speed_range: [f32; 2],
}

impl MovePathSettings {
    const SIZE: usize = 60;
}

/// Instance of a shared group object.
#[binrw]
#[derive(Debug, PartialEq, Default, Clone)]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub struct SharedGroupInstance {
    /// The path to the `.sgb` file.
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
    pub initial_door_state: DoorState,
    #[bw(calc = SharedGroupInstance::SIZE as i32)]
    #[br(temp)]
    overriden_members_offset: i32,
    #[br(temp)]
    #[bw(calc = 0)] // TODO
    overriden_members_count: i32,
    pub initial_rotation_state: RotationState,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub random_timeline_auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub random_timeline_loop_playback: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub collision_controllable_without_eobj: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk_bool: bool,
    pub bound_client_path_instance_id: u32,
    #[bw(calc = SharedGroupInstance::SIZE as i32)]
    #[br(temp)]
    move_path_settings_offset: i32,
    #[brw(pad_before = 4)] // empty, not read
    pub initial_transform_state: TransformState,
    pub initial_color_state: ColourState,
    #[br(restore_position, seek_before = SeekFrom::Current(move_path_settings_offset as i64 - SharedGroupInstance::SIZE as i64))]
    pub move_path_settings: MovePathSettings,
}

impl SharedGroupInstance {
    const SIZE: usize = 92;
}

#[cfg(test)]
mod tests {
    use super::MovePathSettings;
    use crate::common::ensure_size;

    #[test]
    fn test_movepathsettings_size() {
        ensure_size::<MovePathSettings, { MovePathSettings::SIZE }>();
    }
}
