// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common_file_operations::{read_bool_from, write_bool_as};

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum PositionMarkerType {
    DebugZonePop = 0x1,
    DebugJump = 0x2,
    NaviMesh = 0x3,
    LQEvent = 0x4,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct PositionMarkerInstanceObject {
    pub position_marker_type: PositionMarkerType,
    pub comment_jp_offset: u32,
    pub comment_en_offset: u32,
}

#[binrw]
#[brw(repr = u32)]
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum ChairType {
    Chair = 0x0,
    Bed = 0x1,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ChairMarkerInstanceObject {
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    left_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    right_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    back_enable: bool,
    padding: u8,
    chair_type: ChairType,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct QuestMarkerInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct TargetMarkerInstanceObject {}
