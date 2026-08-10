// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::binrw;

use crate::{
    ColorIntensity,
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::StringHeap,
};

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ObjectVisibility {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    /// In "centiseconds".
    pub transition_duration: u16,
    /// Whether this game object should be visible.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)] // padding
    pub visible: bool,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ObjectTransform {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub value: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ObjectOscillator {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub phase_rate: f32,
    pub amplitude: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ObjectRotation {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub value: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ObjectRgbColor {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    color_offset: i32,

    #[br(seek_before = SeekFrom::Current(color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color: ColorIntensity,
}

impl ObjectRgbColor {
    const SIZE: usize = 8;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ObjectRgbColorPair {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    color0_offset: i32,
    color1_offset: i32,

    #[br(seek_before = SeekFrom::Current(color0_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color0: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(color1_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color1: ColorIntensity,
}

impl ObjectRgbColorPair {
    const SIZE: usize = 12;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ObjectRgbaColor {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    color_offset: i32,

    #[br(seek_before = SeekFrom::Current(color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color: ColorIntensity,
}

impl ObjectRgbaColor {
    pub const SIZE: usize = 8;
}
