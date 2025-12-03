// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use super::GameInstanceObject;

#[binrw]
#[derive(Debug, PartialEq)]
#[br(little)]
pub struct AetheryteInstanceObject {
    pub parent_data: GameInstanceObject,
    #[brw(pad_after = 4)] // padding
    pub bound_instance_id: u32,
}
