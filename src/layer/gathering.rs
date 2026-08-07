// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::layer::GameObjectInstanceObject;

/// Gathering point.
///
/// This is stripped out of retail data, and is not used by the client.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct GatheringInstanceObject {
    /// GameObject base ID refers to the GatheringPoint Excel sheet.
    pub parent_data: GameObjectInstanceObject,
}
