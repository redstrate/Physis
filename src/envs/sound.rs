// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::binrw;

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapString, StringHeap},
};

#[binrw]
#[derive(Debug, Default)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct AmbientSoundPaths {
    #[br(temp)]
    #[bw(ignore)]
    heap_pointer: HeapPointer,

    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    offset: u32,
    count: u32,

    #[br(seek_before = SeekFrom::Current(offset as i64 - Self::SIZE as i64))]
    #[br(temp)]
    #[bw(ignore)]
    #[br(restore_position)]
    heap_pointer: HeapPointer,

    #[br(count = count, args { inner: (heap_pointer, string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - Self::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub paths: Vec<HeapString>,
}

impl AmbientSoundPaths {
    pub(crate) const SIZE: usize = 12;
}

#[binrw]
#[derive(Debug, Default)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct AmbientSoundFlags {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub ambient_setting0_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 2)] // padding
    pub ambient_setting1_enabled: bool,
}
