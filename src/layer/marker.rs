// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common_file_operations::{read_bool_from, write_bool_as};

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum PositionMarkerType {
    #[default]
    DebugZonePop = 1,
    DebugJump = 2,
    NaviMesh = 3,
    LQEvent = 4,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct PositionMarkerInstanceObject {
    pub position_marker_type: PositionMarkerType,
    pub comment_jp_offset: u32,
    pub comment_en_offset: u32,
}

#[binrw]
#[brw(repr = u32)]
#[repr(u32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ChairType {
    #[default]
    Chair = 0,
    Bed = 1,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ChairMarkerInstanceObject {
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub left_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub right_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)] // padding, not read
    pub back_enable: bool,
    pub chair_type: ChairType,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct QuestMarkerInstanceObject {
    unk1: u32,
    unk2: u32,
}

#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum TargetMarkerType {
    #[default]
    UiTarget = 0,
    UiNameplate = 1,
    LookAt = 2,
    BodyDyn = 3,
    Root = 4,
    Unk1 = 5, // Seen in bg/ex5/02_ykt_y6/twn/y6t1/level/planevent.lgb
    Unk2 = 6, // Seen in bg/ex5/02_ykt_y6/twn/y6t1/level/planevent.lgb
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TargetMarkerInstanceObject {
    pub nameplate_offset_y: f32,
    pub target_marker_type: TargetMarkerType,
}
