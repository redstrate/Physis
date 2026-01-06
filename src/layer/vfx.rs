// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    layer::Color,
};

#[binrw]
#[derive(Debug, PartialEq)]
pub struct VFXInstanceObject {
    pub asset_path_offset: u32,
    #[brw(pad_after = 4)] // padding
    pub soft_particle_fade_range: f32,
    pub color: Color,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 2)] // padding
    pub no_far_clip: bool,
    pub fade_near_start: f32,
    pub fade_near_end: f32,
    pub fade_far_start: f32,
    pub fade_far_end: f32,
    pub z_correct: f32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct LineVFXInstanceObject {}
