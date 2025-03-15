use binrw::{binread, binrw};

use super::read_bool_from;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum TriggerBoxShape {
    Box = 0x1,
    Sphere = 0x2,
    Cylinder = 0x3,
    Board = 0x4,
    Mesh = 0x5,
    BoardBothSides = 0x6,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct TriggerBoxInstanceObject {
    pub trigger_box_shape: TriggerBoxShape,
    pub priority: i16,
    #[br(map = read_bool_from::<u8>)]
    pub enabled: bool,
    padidng: u8,
    padding1: u32,
}
