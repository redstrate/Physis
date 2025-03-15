// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binread;

use super::GameInstanceObject;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct AetheryteInstanceObject {
    pub parent_data: GameInstanceObject,
    pub bound_instance_id: u32,
    padding: u32,
}
