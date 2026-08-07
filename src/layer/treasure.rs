// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::layer::GameObjectInstanceObject;

/// Treasure object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TreasureInstanceObject {
    pub parent_data: GameObjectInstanceObject,
}
