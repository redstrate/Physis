// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use super::TriggerBoxInstanceObject;

#[binrw]
#[brw(repr = i32)]
#[repr(C)]
#[derive(Debug, PartialEq)]
pub enum ExitType {
    ZoneLine = 0x1,
    Unk = 0x2, // seen in bg/ex5/02_ykt_y6/fld/y6f1/level/planmap.lgb
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(little)]
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
    #[brw(pad_after = 4)] // padding
    pub player_running_direction: f32,
}
