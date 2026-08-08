// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::binrw;

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    layer::GameObjectInstanceObject,
};

/// Base struct for character objects.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct CharacterInstanceObject {
    /// GameObject base ID depends on what kind of character this is.
    pub parent_data: GameObjectInstanceObject,
    // NOTE: Don't remove these, they are needed for correct padding
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
}

/// Event NPC object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EventNpcInstanceObject {
    /// Character base ID refers to the ENpcBase/ENpcResident Excel sheets.
    pub parent_data: CharacterInstanceObject,
    // NOTE: Don't remove these, they are needed for correct padding
    pub unk1: u32,
    /// Sets CharacterLayoutInstance.Unk78
    pub unk2: u32,
    /// Sets EventNpcInstanceObject.Unk80
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk3: bool,
    /// Sets EventNpcInstanceObject.Unk81
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk4: bool,
    /// Sets EventNpcInstanceObject.Unk82
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk5: bool,
    pub unk6: u8,
}

/// Battle NPC object.
///
/// This is stripped out of retail data, and is not used by the client.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct BattleNpcInstanceObject {
    /// Character base ID refers to the BNpcBase Excel sheet.
    pub parent_data: CharacterInstanceObject,
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
    #[br(temp)]
    #[bw(calc = 48)]
    positions_offset: i32,
    #[br(temp)]
    #[bw(calc = positions.len() as i32)]
    positions_count: i32,
    pub horizontal_pop_range: f32,
    pub vertical_pop_range: f32,
    pub bnpc_base_data: i32,
    pub repop_id: u8,
    pub bnpc_rank_id: u8,
    pub territory_range: u16,
    pub bound_instance_id: u32,
    pub fate_layout_label_id: u32,
    pub normal_ai: u32,
    pub server_path_id: u32,
    pub equipment_id: u32,
    pub customize_id: u32,
    #[br(restore_position, seek_before = SeekFrom::Current(positions_offset as i64 - 48), count = positions_count)]
    pub positions: Vec<[f32; 3]>,
}
