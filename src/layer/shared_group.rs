// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::binrw;

use crate::{
    common_file_operations::write_bool_as,
    layer::StringHeap,
    string_heap::{HeapPointer, HeapString},
};

use super::read_bool_from;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum DoorState {
    #[default]
    Auto = 0x1,
    Open = 0x2,
    Closed = 0x3,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum RotationState {
    #[default]
    Rounding = 0x1,
    Stopped = 0x2,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum TransformState {
    #[default]
    Play = 0x0,
    Stop = 0x1,
    Replay = 0x2,
    Reset = 0x3,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum ColourState {
    #[default]
    Play = 0x0,
    Stop = 0x1,
    Replay = 0x2,
    Reset = 0x3,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum MovePathMode {
    #[default]
    None = 0x0,
    SharedGroupAction = 0x1,
    Timeline = 0x2,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum RotationType {
    #[default]
    NoRotate = 0x0,
    AllAxis = 0x1,
    YAxisOnly = 0x2,
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
    pub const SIZE: usize = 60;
}

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
    pub collision_controllable_without_eobj: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk_bool: bool,
    pub bound_client_path_instance_id: u32,
    #[brw(pad_after = 4)] // padding, not read
    #[bw(calc = SharedGroupInstance::SIZE as i32)]
    #[br(temp)]
    move_path_settings_offset: i32,
    pub initial_transform_state: TransformState,
    pub initial_color_state: ColourState,
    #[br(restore_position, seek_before = SeekFrom::Current(move_path_settings_offset as i64 - SharedGroupInstance::SIZE as i64))]
    pub move_path_settings: MovePathSettings,
}

impl SharedGroupInstance {
    pub const SIZE: usize = 92;
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
