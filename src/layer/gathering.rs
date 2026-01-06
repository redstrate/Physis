// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

#[binrw]
#[derive(Debug, PartialEq)]
pub struct GatheringInstanceObject {
    #[brw(pad_after = 4)] // padding
    pub gathering_point_id: u32,
}
