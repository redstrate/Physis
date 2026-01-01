// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use super::common::RelativePositions;

#[binrw]
#[derive(Debug, PartialEq)]
pub struct GameInstanceObject {
    /// For IDs >= 1000000, index into the ENpcBase
    /// For IDs >= 2000000, the row ID reference to EObj/EObjName
    pub base_id: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct NPCInstanceObject {
    pub parent_data: GameInstanceObject,
    pub pop_weather: u32,
    pub pop_time_start: u8,
    #[brw(pad_after = 2)] // padding
    pub pop_time_end: u8,
    pub move_ai: u32,
    pub wandering_range: u8,
    pub route: u8,
    #[brw(pad_after = 8)] // padding
    pub event_group: u16,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ENPCInstanceObject {
    pub parent_data: NPCInstanceObject,
    #[brw(pad_after = 8)] // padding
    pub behavior: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct BNPCInstanceObject {
    pub parent_data: NPCInstanceObject,
    pub name_id: u32,
    pub drop_item: u32,
    pub sense_range_rate: f32,
    pub level: u16,
    pub active_type: u8,
    pub pop_interval: u8,
    pub pop_rate: u8,
    pub pop_event: u8,
    pub link_group: u8,
    pub link_family: u8,
    pub link_range: u8,
    pub link_count_limit: u8,
    pub nonpop_init_zone: u8,
    pub invalid_repop: u8,
    pub link_parent: u8,
    pub link_override: u8,
    pub link_reply: u8,
    pub nonpop: u8,
    pub relative_positions: RelativePositions,
    pub horizontal_pop_range: f32,
    pub vertical_pop_range: f32,
    pub bnpc_base_data: i32,
    pub repop_id: u8,
    pub bnpc_ran_id: u8,
    pub territory_range: u16,
    pub bound_instance_id: u32,
    pub fate_layout_label_id: u32,
    pub normal_ai: u32,
    pub server_path_id: u32,
    pub equipment_id: u32,
    pub customize_id: u32,
    // TODO: read relativeposition data
}
