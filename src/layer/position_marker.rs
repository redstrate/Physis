use binrw::{binread, binrw};

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum PositionMarkerType {
    DebugZonePop = 0x1,
    DebugJump = 0x2,
    NaviMesh = 0x3,
    LQEvent = 0x4,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct PositionMarkerInstanceObject {
    pub position_marker_type: PositionMarkerType,
    pub comment_jp_offset: u32,
    pub comment_en_offset: u32,
}
