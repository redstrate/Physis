// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    layer::RelativePositions,
};

use super::TriggerBoxInstanceObject;

#[binrw]
#[brw(repr = i32)]
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ExitType {
    ZoneLine = 0x1,
    Unk = 0x2, // seen in bg/ex5/02_ykt_y6/fld/y6f1/level/planmap.lgb
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ExitRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    /// What kind of exit range this is.
    pub exit_type: ExitType,
    pub zone_id: u16,
    /// Row ID to TerritoryType that this exit range points to.
    pub territory_type: u16,
    pub index: i32,
    pub destination_instance_id: u32,
    pub return_instance_id: u32,
    #[brw(pad_after = 4)] // padding
    pub player_running_direction: f32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct MapRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    map: u32,
    /// Name for the general location. Index into the PlaceName Sxcel sheet.
    pub place_name_block: u32,
    /// Name for the specific spot. Index into the PlaceName Sxcel sheet.
    pub place_name_spot: u32,
    weather: u32,
    bgm: u32,
    padding: [u8; 10],
    housing_block_id: u8,
    /// Most likely affects whether the EXP bonus affects this area.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub rest_bonus_effective: bool,
    /// Map discovery ID.
    pub discovery_id: u8,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    map_enabled: bool,
    /// Probably to enable indication in the little place name UI element.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub place_name_enabled: bool,
    /// Whether this place is discoverable (see `discovery_id`.)
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub discovery_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    bgm_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    weather_enabled: bool,
    /// Whether this area is marked as a sanctuary.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub rest_bonus_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    bgm_play_zone_in_only: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    lift_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    housing_enabled: bool,
    padding2: [u8; 2],
}

#[binrw]
#[brw(repr = i32)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PopType {
    PC = 0x1,
    Npc = 0x2,
    Content = 0x3,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct PopRangeInstanceObject {
    pub pop_type: PopType,
    pub relative_positions: RelativePositions,
    pub inner_radius_ratio: f32,
    #[brw(pad_after = 7)] // padding
    pub index: u8,
    // TODO: read relative positions
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct EventRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub unk_flags: [u8; 12],
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct FateRangeInstanceObject {}
