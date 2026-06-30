// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common_file_operations::{read_bool_from, write_bool_as};

use super::TriggerBoxInstanceObject;

#[binrw]
#[brw(repr = i32)]
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ExitType {
    #[default]
    ZoneLine = 1,
    Invisible = 2,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
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
#[derive(Debug, PartialEq, Clone, Default)]
pub struct MapRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub map: u32,
    /// Name for the general location. Index into the PlaceName Sxcel sheet.
    pub place_name_block: u32,
    /// Name for the specific spot. Index into the PlaceName Sxcel sheet.
    pub place_name_spot: u32,
    pub weather: u32,
    #[brw(pad_after = 8)] // Not read by the client
    pub bgm: u32,
    pub unk1: u8,
    pub unk2: u8,
    pub housing_block_id: u8,
    /// Most likely affects whether the EXP bonus affects this area.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub rest_bonus_effective: bool,
    /// Map discovery ID.
    pub discovery_id: u8,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub map_enabled: bool,
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
    pub bgm_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub weather_enabled: bool,
    /// Whether this area is marked as a sanctuary.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub rest_bonus_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub bgm_play_zone_in_only: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub lift_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub housing_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub log_flying_height_max_err: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk4: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub mounts_and_ornaments_disabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub lalafells_only: bool,
}

#[binrw]
#[brw(repr = i32)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PopType {
    #[default]
    PC = 0x1,
    Npc = 0x2,
    Content = 0x3,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct PopRangeInstanceObject {
    pub pop_type: PopType,
    // #[br(temp)]
    // #[br(assert(pos == 24))]
    // #[bw(calc = 24)]
    pos: i32,
    #[br(temp)]
    #[bw(calc = positions.len() as i32)]
    pos_count: i32,
    pub inner_radius_ratio: f32, // 16
    pub index: u8,               // 20
    unk1: [u8; 7],
    #[br(count = pos_count)] // NOTE: This is assuming pos is always 24!
    pub positions: Vec<[f32; 3]>,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EventRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct FateRangeInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct GimmickRangeInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ClickableRangeInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct PrefetchRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub bound_instance_id: u32,
    padding: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Clone)]
pub struct DoorRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
}
