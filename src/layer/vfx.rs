// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    Color,
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapString, StringHeap},
};

#[binrw]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct VFXInstanceObject {
    /// Path to an `.avfx` file.
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
    #[brw(pad_after = 4)] // padding, not read
    pub soft_particle_fade_range: f32,
    pub color: Color,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub no_far_clip: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk1: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk2: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk3: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk4: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 1)] // padding, not read
    pub unk5: bool,
    pub fade_near_start: f32,
    pub fade_near_end: f32,
    pub fade_far_start: f32,
    pub fade_far_end: f32,
    pub z_correct: f32,
    pub unk6: u32,
}

#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum LineStyle {
    #[default]
    Red = 1,
    Blue = 2,
    RedFar = 3,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct LineVFXInstanceObject {
    pub line_style: LineStyle,
}
