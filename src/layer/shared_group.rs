use binrw::{binread, binrw};

use super::read_bool_from;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum DoorState {
    Auto = 0x1,
    Open = 0x2,
    Closed = 0x3,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum RotationState {
    Rounding = 0x1,
    Stopped = 0x2,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum TransformState {
    Play = 0x0,
    Stop = 0x1,
    Replay = 0x2,
    Reset = 0x3,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum ColourState {
    Play = 0x0,
    Stop = 0x1,
    Replay = 0x2,
    Reset = 0x3,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct SharedGroupInstance {
    pub asset_path_offset: u32,
    pub initial_door_state: DoorState,
    pub overriden_members: i32,
    pub overriden_members_count: i32,
    pub initial_rotation_state: RotationState,
    #[br(map = read_bool_from::<u8>)]
    pub random_timeline_auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    pub random_timeline_loop_playback: bool,
    #[br(map = read_bool_from::<u8>)]
    pub collision_controllable_without_eobj: bool,
    padding: u8,
    pub bound_client_path_instance_id: u32,
    pub move_path_settings: i32,
    #[br(map = read_bool_from::<u8>)]
    pub not_create_navimesh_door: bool,
    padding1: [u8; 3],
    pub initial_transform_state: TransformState,
    pub initial_color_state: ColourState,
    // TODO: read move path settings
}
