// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::{BinRead, BinResult, BinWrite, binrw};

use crate::string_heap::{HeapPointer, HeapString, StringHeap};

#[binrw::writer(writer, endian)]
pub(crate) fn write_envs(envs: &Vec<EnvsHeader>, string_heap: &mut StringHeap) -> BinResult<()> {
    for env in envs {
        env.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

#[binrw]
#[brw(magic = b"ENVS")]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct EnvsHeader {
    /// Size of this header, in bytes. Should be the same as `EnvsHeader::SIZE`.
    size: u32,
    /// Always 6.
    pub version: u32,
    /// Offset to the sections array.
    offset_to_sections: u32,
    /// Number of sections.
    section_count: u32,
    /// Seems to indicate the remaining amount of bytes in this file, including this u32.
    remaining_size: u32,
    unk1: u32,

    /// List of children sections.
    #[br(count = section_count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset_to_sections as i64 - EnvsHeader::SIZE as i64 + 4))] // Read starting from version
    #[br(restore_position)]
    #[bw(write_with = write_child_sections, args(&mut string_heap,))]
    pub sections: Vec<EnvChildSection>,
}

#[binrw::writer(writer, endian)]
pub(crate) fn write_child_sections(
    sections: &Vec<EnvChildSection>,
    string_heap: &mut StringHeap,
) -> BinResult<()> {
    for section in sections {
        section.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

impl EnvsHeader {
    pub const SIZE: usize = 0x18;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct EnvChildSection {
    offset: u32, // 0xc in size
    count: u32,
    /// Which weather this appleis in. Index into the Weather Excel sheet.
    weather_id: u32,
    offset_to_floats: u32,

    #[br(count = count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub unknown1: Vec<EnvUnknown1>,

    #[br(seek_before = SeekFrom::Current(offset_to_floats as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    floats: [f32; 5],
}

impl EnvChildSection {
    pub const SIZE: usize = 0x10;
}

#[binrw::parser(reader)]
fn unknown2_from_offsets(
    offsets: &Vec<u32>,
    string_heap: &StringHeap,
) -> BinResult<Vec<EnvUnknown2>> {
    let base_offset = reader.stream_position()?;

    let mut layers: Vec<EnvUnknown2> = vec![];

    for offset in offsets {
        let layer_offset = *offset as u64;

        reader.seek(SeekFrom::Start(base_offset + layer_offset))?;
        layers.push(EnvUnknown2::read_le_args(reader, (string_heap,))?); // TODO: don't assume LE
    }

    Ok(layers)
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct EnvUnknown1 {
    offset: u32,
    count: u32,
    /// Up to 14, I think.
    index: u32,

    #[br(count = count)]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvUnknown1::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub unknown2_offsets: Vec<u32>,

    // TODO: This is bad, we are failing to parse part of bgcommon/env/local/ffxiv_lenv/lenv_sea_s1/lenv_s1_cave.envb.
    // This is just a workaround.
    #[br(try, parse_with = unknown2_from_offsets, args(&unknown2_offsets, string_heap))]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvUnknown1::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub unknown2: Vec<EnvUnknown2>,
}

impl EnvUnknown1 {
    pub const SIZE: usize = 0xc;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct EnvUnknown2 {
    // unconfirmed
    pub unk1: f32,
    pub unk2: f32,
    pub unk3: f32,
    pub unk4: f32,
    pub unk5: f32,
    pub unk6: f32,
    pub unk7: f32,
    pub unk8: f32,

    pub unk9: f32,
    pub unk10: f32,
    pub unk11: f32,
    pub unk12: f32,
    #[br(temp)]
    #[bw(ignore)]
    heap_pointer: HeapPointer,
    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub name: HeapString,
    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub name2: HeapString,
    pub unk15: f32,
    pub unk16: f32,
    pub unk17: f32,
    pub unk18: f32,
    pub unk19: f32,
    pub unk20: f32,
    pub unk21: f32,
    pub unk22: f32,
    pub unk23: f32,
    pub unk24: f32,
}

impl EnvUnknown2 {
    pub const SIZE: usize = 0x20; // unconfirmed
}
