// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapString, StringHeap},
};

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(C)]
pub enum SoundEffectType {
    #[default]
    Point = 0x3,
    PointDir = 0x4,
    Line = 0x5,
    PolyLine = 0x6,
    Surface = 0x7,
    BoardObstruction = 0x8,
    BoxObstruction = 0x9,
    PolyLineObstruction = 0xB,
    PolygonObstruction = 0xC,
    LineExtController = 0xD,
    Polygon = 0xE,
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct SoundParameters {
    pub sound_effect_type: SoundEffectType,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_no_far_clip: bool,
    pub unk1: u16, // Padding according to Lumina
    pub binary_offset: i32,
    pub binary_count: i32,
    pub point_selection: u32,

    #[br(count = binary_count)]
    pub binaries: Vec<u8>,
}

#[binrw]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct SoundInstanceObject {
    // I think it's always 56 so...
    #[bw(calc = 56)]
    #[br(temp)]
    parameters_offset: i32,
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
    // NOTE: This is assuming `parameters_offset` is always 56! (48 is not included)
    pub parameters: SoundParameters,
}
