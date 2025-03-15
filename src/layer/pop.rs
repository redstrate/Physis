use binrw::{binread, binrw};

use super::common::RelativePositions;

#[binread]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum PopType {
    PC = 0x1,
    Npc = 0x2,
    Content = 0x3,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct PopRangeInstanceObject {
    pub pop_type: PopType,
    pub relative_positions: RelativePositions,
    pub inner_radius_ratio: f32,
    pub index: u8,
    padding1: [u8; 3],
    padding2: u32,
    // TODO: read relative positions
}
