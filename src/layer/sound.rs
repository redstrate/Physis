// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::string_heap::{HeapPointer, HeapString, StringHeap};

#[binrw]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, PartialEq)]
pub struct SoundInstanceObject {
    pub sound_effect_param: i32,
    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub asset_path: HeapString,
    // TODO: read separam
}
