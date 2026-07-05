// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use super::GameObjectInstanceObject;

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct AetheryteInstanceObject {
    pub parent_data: GameObjectInstanceObject,
    pub bound_instance_id: u32,
    unk1: u32,
}
