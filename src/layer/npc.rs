use binrw::{binread, binrw};

use super::common::RelativePositions;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct GameInstanceObject {
    base_id: u32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct NPCInstanceObject {
    parent_data: GameInstanceObject,
    pop_weather: u32,
    pop_time_start: u8,
    pop_time_end: u8,
    padding: u16,
    move_ai: u32,
    wandering_range: u8,
    route: u8,
    event_group: u16,
    padding1: [u32; 2],
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ENPCInstanceObject {
    pub parent_data: NPCInstanceObject,
    pub behavior: u32,
    padding: [u32; 2],
}

#[binread]
#[derive(Debug)]
#[br(little)]
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
