// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::{BinRead, BinResult, BinWrite, binrw};

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapString, StringHeap},
};

pub(crate) const DAWNTRAIL_MARKER: &[u8; 4] = b"007V";

pub(crate) fn read_dawntrail_marker(x: [u8; 4]) -> bool {
    &x == DAWNTRAIL_MARKER
}

pub(crate) fn write_dawntrail_marker(x: &bool) -> [u8; 4] {
    if *x { *DAWNTRAIL_MARKER } else { [0; 4] }
}

#[binrw::writer(writer, endian)]
pub(crate) fn write_envs(envs: &Vec<Envs>, string_heap: &mut StringHeap) -> BinResult<()> {
    for env in envs {
        env.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

/// ENVS section used in some files.
#[binrw]
#[brw(magic = b"ENVS")]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
pub struct Envs {
    /// Size of this header, in bytes. Should be the same as [EnvsHeader::SIZE].
    size: u32,
    /// The client doesn't load anything but version 6.
    pub version: u32,
    /// Offset to the sections array.
    #[br(temp)]
    #[bw(calc = 16)]
    offset_to_sections: u32,
    /// Number of sections.
    #[br(temp)]
    #[bw(calc = sections.len() as u32)]
    section_count: u32,
    /// Seems to indicate the remaining amount of bytes in this file, including this u32.
    remaining_size: u32,
    /// Equal to `section_count` * `EnvChildSection::SIZE`.
    #[br(temp)]
    #[bw(calc = section_count * EnvChildSection::SIZE as u32)]
    section_size: u32,

    /// List of children sections.
    #[br(count = section_count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset_to_sections as i64 - Envs::SIZE as i64 + 4))] // Read starting from version
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

impl Envs {
    pub(crate) const SIZE: usize = 0x18;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
pub struct EnvChildSection {
    #[br(temp)]
    #[bw(calc = 0)] // TODO
    offset: u32,

    #[br(temp)]
    #[bw(calc = 0)] // TODO
    count: u32,
    /// Which weather this applied in. Index into the Weather Excel sheet.
    pub weather_id: u32,
    #[br(temp)]
    #[bw(calc = 0)] // TODO
    offset_to_floats: u32,

    #[br(count = count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub timelines: Vec<EnvTimeline>,

    #[br(seek_before = SeekFrom::Current(offset_to_floats as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    floats: [f32; 5],
}

impl EnvChildSection {
    pub(crate) const SIZE: usize = 0x10;
}

#[binrw::parser(reader, endian)]
fn unknown2_from_offsets<T>(
    size: u32,
    main_offset: u32,
    offsets: &[i32],
    string_heap: &StringHeap,
) -> BinResult<Vec<T>>
where
    T: for<'a> BinRead<Args<'a> = (&'a StringHeap,)>,
{
    let base_offset = reader.stream_position()? - size as u64;

    let mut layers: Vec<T> = vec![];

    for offset in offsets {
        let layer_offset = *offset as u64;

        reader.seek(SeekFrom::Start(
            base_offset + (layer_offset + main_offset as u64),
        ))?;
        layers.push(T::read_options(reader, endian, (string_heap,))?);
    }

    Ok(layers)
}

#[binrw]
#[br(import(index: u32, offset: u32, unknown2_offsets: &[i32], string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
pub enum EnvTimelineElement {
    #[br(pre_assert(index == 0))]
    Element0(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element0>,
    ),
    #[br(pre_assert(index == 1))]
    Element1(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element1>,
    ),
    #[br(pre_assert(index == 2))]
    Element2(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element2>,
    ),
    #[br(pre_assert(index == 3))]
    Element3(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element3>,
    ),
    #[br(pre_assert(index == 4))]
    Element4(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element4>,
    ),
    #[br(pre_assert(index == 6))]
    Element6(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element6>,
    ),
    #[br(pre_assert(index == 7))]
    Element7(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element7>,
    ),
    #[br(pre_assert(index == 8))]
    Element8(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element8>,
    ),
    #[br(pre_assert(index == 9))]
    Element9(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element9>,
    ),
    #[br(pre_assert(index == 10))]
    Element10(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element10>,
    ),
    #[br(pre_assert(index == 11))]
    Element11(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element11>,
    ),
    #[br(pre_assert(index == 12))]
    Element12(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element12>,
    ),
    #[br(pre_assert(index == 13))]
    Element13(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element13>,
    ),
    #[br(pre_assert(index == 20))]
    Element20(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element20>,
    ),
    #[br(pre_assert(index == 29))]
    ChangeVisibility(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<ChangeVisibility>,
    ),
    #[br(pre_assert(index == 31))]
    Element31(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element31>,
    ),
    #[br(pre_assert(index == 33))]
    Element33(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element33>,
    ),
    #[br(pre_assert(index == 34))]
    Element34(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element34>,
    ),
    #[br(pre_assert(index == 35))]
    Element35(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Element35>,
    ),
    #[default] // TODO: is this is a sensible default?
    UnknownNeedsParsing,
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
pub struct EnvTimeline {
    #[br(temp)]
    #[bw(ignore)]
    offset: u32,
    #[br(temp)]
    #[bw(ignore)]
    count: u32,
    index: u32,

    // NOTE: The size of array elements *must* match the distance between offsets!
    // If there's a discrepancy, that means our struct size is wrong.
    #[br(count = count)]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvTimeline::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    #[br(temp)]
    offsets: Vec<i32>,

    #[br(args(index, offset, &offsets, string_heap))]
    #[bw(args(string_heap))]
    pub data: EnvTimelineElement,
}

impl EnvTimeline {
    pub(crate) const SIZE: usize = 0xc;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element0 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: f32,
    unk4: f32,
    unk5: f32,
    unk6: u32,
    unk7: u32,
    unk8: f32,
    unk9: f32,
    unk10: f32,
    unk11: f32,
    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    unk13: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element1 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: f32,
    unk6: f32,
    unk7: f32,
    unk8: u32,
    unk9: f32,
    unk10: f32,
    unk11: f32,
    unk12: f32,
    unk13: f32,
    unk14: [u8; 3],
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element2 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: f32,
    unk6: u32,
    unk7: u32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element3 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: u32,
    unk9: u32,
    unk10: u32,
    unk11: u32,
    unk12: u32,
    unk13: u32,
    unk14: u32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element4 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: u32,
    unk9: u32,
    unk10: u32,
    unk11: u32,
    unk12: u32,
    unk13: u32,
    unk14: u32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element6 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: f32,
    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    unk5: f32,
    unk7: f32,
    unk8: f32,
    unk9: f32,
    unk10: u32,
    unk11: u32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element7 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: u32,
    unk9: u32,
    unk10: u32,
    unk11: u32,
    unk12: u32,
    unk13: u32,
    unk14: u32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element8 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: f32,
    unk3: f32,
    unk4: f32,
    unk5: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element9 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: f32,
    unk3: f32,
    unk4: f32,
    unk5: f32,
    unk6: f32,
    unk7: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element10 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: f32,
    unk4: u32,
    unk5: f32,
    unk6: u32,
    unk7: u32,
    unk8: f32,
    unk9: u32,
    unk10: u32,
    unk11: u32,
    unk12: f32,
    unk13: u32,
    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    unk15: u32,
    unk16: u32,
    unk17: u32,
    unk18: u32,
    unk19: u32,
    unk20: f32,
    unk21: f32,
    unk22: f32,
    unk23: f32,
}

#[binrw]
#[derive(Debug, Default)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element11 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    offset: u32,
    count: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: f32,
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: f32,
    unk9: f32,
    unk10: u32,
    unk11: u32,
    unk12: f32,
    unk13: f32,
    unk14: f32,
    unk15: f32,
    unk16: f32,
    unk17: f32,
    unk18: f32,
    unk19: f32,

    #[br(seek_before = SeekFrom::Current(offset as i64 - Element11::SIZE as i64))]
    #[br(temp)]
    #[bw(ignore)]
    #[br(restore_position)]
    heap_pointer: HeapPointer,

    #[br(count = count, args { inner: (heap_pointer, string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - Element11::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub paths: Vec<HeapString>,
}

impl Element11 {
    pub(crate) const SIZE: usize = 0x58;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element12 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: u32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element13 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: f32,
    unk4: f32,
    unk5: f32,
    unk6: f32,
    unk7: f32,
    unk8: f32,
    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    unk10: f32,
    unk11: f32,
    unk12: f32,
    unk13: f32,
    unk14: f32,
    unk15: f32,
    unk16: f32,
    unk17: f32,
    unk18: f32,
    unk19: f32,
    unk20: f32,
    unk21: f32,
    unk22: u8,
}

#[binrw]
#[derive(Debug, Default)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element20 {
    #[br(temp)]
    #[bw(ignore)]
    heap_pointer: HeapPointer,

    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    offset: u32,
    count: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: u32,

    #[br(seek_before = SeekFrom::Current(offset as i64 - Element20::SIZE as i64))]
    #[br(temp)]
    #[bw(ignore)]
    #[br(restore_position)]
    heap_pointer: HeapPointer,

    #[br(count = count, args { inner: (heap_pointer, string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - Element20::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub paths: Vec<HeapString>,
}

impl Element20 {
    pub(crate) const SIZE: usize = 0x20;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ChangeVisibility {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    // Always seems to be 200?
    unk2: u16,
    /// Whether this game object should be visible.
    #[br(map = read_bool_from::<u16>)]
    #[bw(map = write_bool_as::<u16>)]
    pub visible: bool,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element31 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: f32,
    unk3: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element33 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element34 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: f32,
    unk5: f32,
    unk6: f32,
    unk7: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Element35 {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    unk3: u32,
    unk4: f32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_envheader_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<EnvsHeader, { EnvsHeader::SIZE }>();
    }

    #[test]
    fn test_envchildsection_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<EnvChildSection, {EnvChildSection::SIZE }>();
    }

    #[test]
    fn test_envunknown1_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<EnvUnknown1, { EnvUnknown1::SIZE }>();
    }

    #[test]
    fn test_element20_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<Element11, { Element11::SIZE }>();
    }

    #[test]
    fn test_element11_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<Element20, { Element20::SIZE }>();
    }
}
