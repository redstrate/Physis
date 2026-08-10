// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::{BinRead, BinResult, BinWrite, binrw};

use crate::{common_file_operations::read_string_until_null, string_heap::StringHeap};

mod environment;
pub use environment::*;

mod object;
pub use object::*;

mod sound;
pub use sound::*;

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
    offset_to_sections: i32,
    /// Number of sections.
    #[br(temp)]
    #[bw(calc = sections.len() as u32)]
    section_count: u32,
    auxiliary_offset: i32, // TODO: read this data
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
fn write_child_sections(
    sections: &Vec<EnvChildSection>,
    string_heap: &mut StringHeap,
) -> BinResult<()> {
    for section in sections {
        section.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

impl Envs {
    const SIZE: usize = 0x18;
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
    /// I think which weather this applied in. Index into the Weather Excel sheet.
    pub owner_id: u32,
    #[br(temp)]
    #[bw(calc = 0)] // TODO
    offset_to_footer: u32,

    #[br(count = count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)]
    pub timelines: Vec<EnvTimeline>,

    /// In seconds.
    #[br(seek_before = SeekFrom::Current(offset_to_footer as i64 - EnvChildSection::SIZE as i64), restore_position)]
    pub cycle_length: f32,
    #[br(seek_before = SeekFrom::Current(offset_to_footer as i64 + 4 - EnvChildSection::SIZE as i64), restore_position)]
    pub section_parameter: u32,
    #[br(seek_before = SeekFrom::Current(offset_to_footer as i64 + 8 - EnvChildSection::SIZE as i64), restore_position)]
    pub section_weight: f32,
    #[br(seek_before = SeekFrom::Current(offset_to_footer as i64 + 12 - EnvChildSection::SIZE as i64), restore_position)]
    resource_path_0_offset: i32,
    #[br(seek_before = SeekFrom::Current(offset_to_footer as i64 + 16 - EnvChildSection::SIZE as i64), restore_position)]
    resource_path_1_offset: i32,

    #[br(parse_with = read_string_until_null, seek_before = SeekFrom::Current(offset_to_footer as i64 + resource_path_0_offset as i64 - EnvChildSection::SIZE as i64), restore_position)]
    #[bw(ignore)]
    pub resource_path_0: String,
    #[br(parse_with = read_string_until_null, seek_before = SeekFrom::Current(offset_to_footer as i64 + resource_path_1_offset as i64 - EnvChildSection::SIZE as i64), restore_position)]
    #[bw(ignore)]
    pub resource_path_1: String,
}

impl EnvChildSection {
    const SIZE: usize = 0x10;
}

#[binrw::parser(reader, endian)]
fn unknown2_from_offsets<T>(
    size: u32,
    main_offset: i32,
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
#[br(import(index: u32, offset: i32, unknown2_offsets: &[i32], string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
pub enum EnvTimelineElement {
    #[br(pre_assert(index == 0))]
    GlobalLighting(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<GlobalLighting>,
    ),
    #[br(pre_assert(index == 1))]
    FakeSpecular(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<FakeSpecular>,
    ),
    #[br(pre_assert(index == 2))]
    Cloud(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<Cloud>,
    ),
    #[br(pre_assert(index == 3))]
    Rain(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<WeatherParticles>,
    ),
    #[br(pre_assert(index == 4))]
    Snow(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<WeatherParticles>,
    ),
    #[br(pre_assert(index == 5))]
    Dust(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<WeatherParticles>,
    ),
    #[br(pre_assert(index == 6))]
    Wind(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<Wind>,
    ),
    #[br(pre_assert(index == 7))]
    LightShaft(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<LightShaft>,
    ),
    #[br(pre_assert(index == 8))]
    Wetness(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<Wetness>,
    ),
    #[br(pre_assert(index == 9))]
    ToneMapping(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ToneMapping>,
    ),
    #[br(pre_assert(index == 10))]
    ColorFilter(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ColorFilter>,
    ),
    #[br(pre_assert(index == 11))]
    Effect(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<Effect>,
    ),
    #[br(pre_assert(index == 12))]
    Starfield(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<Starfield>,
    ),
    #[br(pre_assert(index == 13))]
    VerticalFog(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<VerticalFog>,
    ),
    #[br(pre_assert(index == 20))]
    AmbientSoundPaths(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<AmbientSoundPaths>,
    ),
    #[br(pre_assert(index == 21))]
    AmbientSoundFlags(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<AmbientSoundFlags>,
    ),
    #[br(pre_assert(index == 29))]
    ObjectVisibility(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ObjectVisibility>,
    ),
    #[br(pre_assert(index == 30))]
    ObjectTransform(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ObjectTransform>,
    ),
    #[br(pre_assert(index == 31))]
    ObjectOscillator(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ObjectOscillator>,
    ),
    #[br(pre_assert(index == 32))]
    ObjectRotation(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ObjectRotation>,
    ),
    #[br(pre_assert(index == 33))]
    ObjectRgbColor(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ObjectRgbColor>,
    ),
    #[br(pre_assert(index == 34))]
    ObjectRgbColorPair(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ObjectRgbColorPair>,
    ),
    #[br(pre_assert(index == 35))]
    ObjectRgbaColor(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)]
        Vec<ObjectRgbaColor>,
    ),
    #[default]
    Unknown,
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
pub struct EnvTimeline {
    #[br(temp)]
    #[bw(ignore)]
    offset: i32,
    #[br(temp)]
    #[bw(ignore)]
    count: u32,
    index: u32,

    // NOTE: The size of array elements *must* match the distance between offsets!
    // If there's a discrepancy, that means our struct size is wrong.
    #[br(count = count)]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvTimeline::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)]
    #[br(temp)]
    offsets: Vec<i32>,

    #[br(args(index, offset, &offsets, string_heap))]
    #[bw(args(string_heap))]
    pub data: EnvTimelineElement,
}

impl EnvTimeline {
    const SIZE: usize = 0xc;
}
