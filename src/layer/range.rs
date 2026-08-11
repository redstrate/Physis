// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::binrw;

use crate::common_file_operations::{read_bool_from, write_bool_as};

use super::TriggerBoxInstanceObject;

#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ExitType {
    #[default]
    ZoneLine = 1,
    Invisible = 2,
}

/// Zone transitions (the visible part is probably LineVFX?)
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
    pub player_running_direction: f32,
    #[brw(pad_after = 2)] // padding, not read
    pub unk9c: u16,
    pub unk_instance_id: u32,
}

/// Used to demarcate various aspects of the map, such as location or the BGM used.
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
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PopType {
    #[default]
    PC = 1,
    Npc = 2,
    Content = 3,
}

/// Generic range for characters to spawn in.
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

/// Generic areas for events to use like FATEs or cutscene triggers.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EventRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
}

/// Unknown object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct FateRangeInstanceObject {
    pub parent_data: RangeInstanceObject,
    #[brw(pad_before = 8)] // padding, not read
    /// ???
    pub fate_layout_label_id: u32,
}

/// Unknown object.
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

/// Unknown object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ClickableRangeInstanceObject {
    pub parent_data: RangeInstanceObject,
}

/// A hint for the client to preload an area.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct PrefetchRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub bound_instance_id: u32,
}

/// A hint used to animate the opening of doors.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct DoorRangeInstanceObject {
    pub parent_data: RangeInstanceObject,
}

#[binrw]
#[repr(u32)]
#[brw(repr = u32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum RangeShape {
    #[default]
    None,
    Box,
    Sphere,
    Cylinder,
    Plane,
}

/// Base struct for range objects.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct RangeInstanceObject {
    pub shape: RangeShape,
}

/// Provides underwater effects like bubbles and swimming.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct WaterRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    /// If false, the underwater effect is not visible and you don't swim if spawned inside.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub enabled: bool,
    /// This doesn't seem to do anything? It's definitely read.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk2: bool,
}

/// Unknown object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ShowHideRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub unk_offset: i32,
    pub unk_count: u32,
}

/// Unknown object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct GameContentsRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
}

/// Unknown object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EventEffectRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
}
