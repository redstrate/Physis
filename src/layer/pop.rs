// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binread;

use super::common::RelativePositions;

#[binread]
#[brw(repr = i32)]
#[repr(C)]
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
    #[brw(pad_after = 7)] // padding
    pub index: u8,
    // TODO: read relative positions
}
