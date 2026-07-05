// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

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
    unk1: u32,
    #[brw(pad_after = 2)] // padding, not read
    unk2: u16,
    unk3: u32,
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
    #[br(temp)]
    #[bw(calc = 12)]
    pos: i32,
    #[br(temp)]
    #[bw(calc = positions.len() as i32)]
    pos_count: i32,
    pub inner_radius_ratio: f32,
    // Start reading based from `pos`.
    #[br(restore_position, seek_before = SeekFrom::Current(pos as i64 - 12), count = pos_count)]
    pub positions: Vec<[f32; 3]>,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EventRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct FateRangeInstanceObject {
    pub parent_data: RangeInstanceObject,
    #[brw(pad_before = 8)] // padding, not read
    unk1: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct GimmickRangeInstanceObject {
    pub parent_data: RangeInstanceObject,
    unk1: u16,
    #[brw(pad_after = 5)] // padding, not read
    unk2: u8, // boolean probably
    unk3: u32,
    #[brw(pad_after = 4)] // padding, not read
    unk4: u32,
    unk5: u16,
    unk6: u16,
    unk7: u16,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ClickableRangeInstanceObject {
    pub parent_data: RangeInstanceObject,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct PrefetchRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub bound_instance_id: u32,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct DoorRangeInstanceObject {
    pub parent_data: RangeInstanceObject,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct RangeInstanceObject {
    unk1: i32,
}
