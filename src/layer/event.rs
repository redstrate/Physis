// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::layer::GameObjectInstanceObject;

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct EventObjectInstanceObject {
    pub parent_data: GameObjectInstanceObject,
    /// A reference to another object, most likely.
    #[brw(pad_after = 4)] // padding, not read
    pub bound_instance_id: u32,
    pub unk1: u8, // boolean I think
}
