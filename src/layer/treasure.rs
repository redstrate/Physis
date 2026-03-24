// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

#[binrw]
#[derive(Debug, PartialEq)]
pub struct TreasureInstanceObject {
    #[brw(pad_after = 11)] // padding
    /// Index into the Treasure Excel sheet.
    pub base_id: u8,
}
