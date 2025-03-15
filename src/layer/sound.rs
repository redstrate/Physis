// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct SoundInstanceObject {
    pub sound_effect_param: i32,
    pub asset_path_offset: u32,
    // TODO: read separam
}
