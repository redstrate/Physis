// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{binread, binrw};

use super::TriggerBoxInstanceObject;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum ExitType {
    ZoneLine = 0x1,
    Unk = 0x2, // seen in bg/ex5/02_ykt_y6/fld/y6f1/level/planmap.lgb
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ExitRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub exit_type: ExitType,
    pub zone_id: u16,
    pub territory_type: u16,
    pub index: i32,
    pub destination_instance_id: u32,
    pub return_instance_id: u32,
    #[brw(pad_after = 4)] // padding
    pub player_running_direction: f32,
}
