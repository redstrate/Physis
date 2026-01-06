// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::layer::GameInstanceObject;

#[binrw]
#[derive(Debug, PartialEq)]
pub struct EventInstanceObject {
    pub parent_data: GameInstanceObject,
    /// A reference to another object, most likely.
    pub bound_instance_id: u32,
    #[brw(pad_after = 8)] // padding?
    pub linked_instance_id: u32,
}
