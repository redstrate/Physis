// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::layer::GameObjectInstanceObject;

/// Event object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EventObjectInstanceObject {
    /// GameObject base ID refers to the EObj Excel sheet.
    pub parent_data: GameObjectInstanceObject,
    /// A reference to another object, most likely.
    #[brw(pad_after = 4)] // padding, not read
    pub bound_instance_id: u32,
    #[brw(pad_after = 7)] // don't think is read but needed to correct to the right size
    pub unk1: u8, // boolean I think
}
