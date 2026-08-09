// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::{BinRead, BinResult, BinWrite, binrw};

use crate::{
    ColorIntensity,
    common_file_operations::{
        read_bool_from, read_dawntrail_marker, read_string_until_null, write_bool_as,
        write_dawntrail_marker,
    },
    string_heap::{HeapPointer, HeapString, StringHeap},
};

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
    pub owner_id: u32,
    #[br(temp)]
    #[bw(calc = 0)] // TODO
    offset_to_footer: u32,

    #[br(count = count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
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
    GlobalLighting(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<GlobalLighting>,
    ),
    #[br(pre_assert(index == 1))]
    FakeSpecular(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<FakeSpecular>,
    ),
    #[br(pre_assert(index == 2))]
    Cloud(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Cloud>,
    ),
    #[br(pre_assert(index == 3))]
    Rain(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<WeatherParticles>,
    ),
    #[br(pre_assert(index == 4))]
    Snow(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<WeatherParticles>,
    ),
    #[br(pre_assert(index == 5))]
    Dust(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<WeatherParticles>,
    ),
    #[br(pre_assert(index == 6))]
    Wind(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Wind>,
    ),
    #[br(pre_assert(index == 7))]
    LightShaft(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<LightShaft>,
    ),
    #[br(pre_assert(index == 8))]
    Wetness(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Wetness>,
    ),
    #[br(pre_assert(index == 9))]
    ToneMapping(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<ToneMapping>,
    ),
    #[br(pre_assert(index == 10))]
    ColorFilter(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<ColorFilter>,
    ),
    #[br(pre_assert(index == 11))]
    Effect(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Effect>,
    ),
    #[br(pre_assert(index == 12))]
    Starfield(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<Starfield>,
    ),
    #[br(pre_assert(index == 13))]
    VerticalFog(
        #[br(parse_with = unknown2_from_offsets, args(EnvTimeline::SIZE as u32, offset, unknown2_offsets, string_heap))]
        #[br(restore_position)]
        #[bw(ignore)] // TODO: support writing
        Vec<VerticalFog>,
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
pub struct GlobalLighting {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    sunlight_color_offset: u32,
    pub ambient_light_scale: f32,
    pub ambient_light_saturation: f32,
    pub ambient_attenuation: f32,
    extra_ambient_color_offset: u32,
    moonlight_color_offset: u32,
    pub extra_ambient_color_weight: f32,
    pub extra_param: f32,
    pub param0: f32,
    pub param1: f32,

    #[br(seek_before = SeekFrom::Current(sunlight_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub sunlight_color: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(extra_ambient_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub extra_ambient_color: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(moonlight_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub moonlight_color: ColorIntensity,

    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    #[br(if(is_dawntrail))]
    pub hue_shift: f32,
}

impl GlobalLighting {
    pub const SIZE: usize = 52;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct FakeSpecular {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    color0_offset: u32,
    color1_offset: u32,
    color2_offset: u32,
    /// In degrees.
    pub elevation0: f32,
    /// In degrees.
    pub elevation1: f32,
    /// In degrees.
    pub elevation2: f32,
    /// In degrees.
    pub rotation: f32,

    #[br(seek_before = SeekFrom::Current(color0_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color0: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(color1_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color1: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(color2_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color2: ColorIntensity,
}

impl FakeSpecular {
    pub const SIZE: usize = 32;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Cloud {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub main_cloud: u32,
    pub alternate_cloud: u32,
    pub main_intensity: f32,
    pub alternate_intensity: f32,
    diffuse_color_offset: u32,
    ambient_color_offset: u32,

    #[br(seek_before = SeekFrom::Current(diffuse_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub diffuse_color: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(ambient_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub ambient_color: ColorIntensity,
}

impl Cloud {
    pub const SIZE: usize = 28;
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
pub struct WeatherParticles {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub density: f32,
    pub oscillation_spread: f32,
    pub oscillation_frequency: f32,
    pub distance_response_profile: f32,
    pub extra_param: f32,
    pub modulation_rate: f32,
    color_offset: u32,
    pub flags: u32,

    #[br(seek_before = SeekFrom::Current(color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color: ColorIntensity,
}

impl WeatherParticles {
    pub const SIZE: usize = 36;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Wind {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    /// In degrees.
    pub layer0_azimuth: u32,
    pub unk3: f32,
    pub layer0_max_strength: f32,
    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    /// In degrees.
    #[br(if(is_dawntrail))]
    pub layer1_azimuth: f32,
    #[br(if(is_dawntrail))]
    pub layer0_wavelength: f32,
    #[br(if(is_dawntrail))]
    pub layer1_max_strength: f32,
    #[br(if(is_dawntrail))]
    pub layer1_wavelength: f32,
    #[br(if(is_dawntrail))]
    pub layer0_min_strength: f32,
    #[br(if(is_dawntrail))]
    pub layer1_min_strength: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct LightShaft {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    unk2: u32,
    color_offset: u32,
    radiance_color_offset: u32,
    pub scale: f32,
    pub param: f32,

    #[br(seek_before = SeekFrom::Current(color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub color: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(radiance_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub radiance_color: ColorIntensity,
}

impl LightShaft {
    pub const SIZE: usize = 24;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Wetness {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub world_wetness_parameter1: f32,
    pub world_wetness_parameter0: f32,
    pub character_wetness: f32,
    pub unk5: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ToneMapping {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub adaptation_rate: f32,
    pub adapted_luminance_parameter_w: f32,
    pub adapted_luminance_parameter_x: f32,
    pub adapted_luminance_parameter_y: f32,
    pub tone_map_parameter_y: f32,
    pub tone_map_parameter_x: f32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct ColorFilter {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub hue: f32,
    pub saturation: f32,
    pub brightness: f32,
    pub contrast: f32,
    filter_color_offset: i32,
    pub filter_intensity: f32,
    pub sepia: f32,
    pub grayscale: f32,
    pub negative: f32,
    pub lut_input_black_point: f32,
    pub lut_input_white_point: f32,
    #[brw(pad_after = 3)] // padding
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub alternate_curve_layout: bool,

    #[br(seek_before = SeekFrom::Current(filter_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub filter_color: ColorIntensity,

    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    #[br(if(is_dawntrail))]
    pub dark_filter_saturation: f32,
    #[br(if(is_dawntrail))]
    pub dark_filter_parameter_x: f32,
    #[br(if(is_dawntrail))]
    pub dark_filter_parameter_y: f32,
    #[br(if(is_dawntrail))]
    pub dark_filter_tint_amount_and_parameter_z: f32,
    #[br(if(is_dawntrail))]
    dark_filter_tint_color_offset: i32,

    #[br(if(is_dawntrail), seek_before = SeekFrom::Current(dark_filter_tint_color_offset as i64 - Self::DAWNTRAIL_SIZE as i64), restore_position)]
    pub dark_filter_tint_color: ColorIntensity,
}

impl ColorFilter {
    pub const SIZE: usize = 52;
    pub const DAWNTRAIL_SIZE: usize = Self::SIZE + 24;
}

#[binrw]
#[derive(Debug, Default)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Effect {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    path_offset: i32,
    path_count: u32,
    background_tint_color_offset: i32,
    foreground_tint_color_offset: i32,
    #[brw(pad_after = 3)] // padding
    pub foreground_effect_type: u8,
    pub effect_transition_seconds: f32,
    unk_color_offset: i32,
    thunder_color_offset: i32,
    pub thunder_interval: u32,
    pub background_intensity: f32,
    pub foreground_intensity: f32,

    #[br(seek_before = SeekFrom::Current(path_offset as i64 - Self::SIZE as i64))]
    #[br(temp)]
    #[bw(ignore)]
    #[br(restore_position)]
    heap_pointer: HeapPointer,

    #[br(count = path_count, args { inner: (heap_pointer, string_heap,) })]
    #[br(seek_before = SeekFrom::Current(path_offset as i64 - Self::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub paths: Vec<HeapString>,

    #[br(seek_before = SeekFrom::Current(background_tint_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub background_tint_color: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(foreground_tint_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub foreground_tint_color: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(unk_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub unk_color: ColorIntensity,

    #[br(seek_before = SeekFrom::Current(thunder_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub thunder_color: ColorIntensity,
}

impl Effect {
    pub(crate) const SIZE: usize = 48;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct Starfield {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    pub a_intensity: u32,
    pub b_intensity: u32,
    pub c_intensity: u32,
    pub unk5: f32,
    pub moon_color_offset: u32,
    pub unk7: f32,
    pub procedural_star_intensity: f32,

    #[br(seek_before = SeekFrom::Current(moon_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub moon_color: ColorIntensity,
}

impl Starfield {
    pub const SIZE: usize = 32;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct VerticalFog {
    /// Between 0 and 86400 seconds (one day.)
    pub time: f32,
    fog_color_offset: u32,
    pub fog_start_distance: f32,
    pub fog_intensity_0: f32,
    pub fog_fade_distance: f32,
    pub fog_intensity_1: f32,
    pub fog_parameter: f32,
    pub fog_blend: f32,

    #[br(seek_before = SeekFrom::Current(fog_color_offset as i64 - Self::SIZE as i64), restore_position)]
    pub fog_color: ColorIntensity,

    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,
    /// As a percent.
    #[br(if(is_dawntrail))]
    pub fog_density: f32,
    #[br(if(is_dawntrail))]
    pub exp_fog_height: f32,
    #[br(if(is_dawntrail))]
    pub fog_height_falloff: f32,
    #[br(if(is_dawntrail))]
    pub start_distance: f32,
    #[br(if(is_dawntrail))]
    pub fog_min_opacity: f32,
    #[br(if(is_dawntrail))]
    pub fog_density_2_percent: f32,
    #[br(if(is_dawntrail))]
    pub exp_fog_height_2_delta: f32,
    #[br(if(is_dawntrail))]
    pub fog_height_falloff_2: f32,
    #[br(if(is_dawntrail))]
    pub directional_inscattering_start_distance: f32,
    #[br(if(is_dawntrail))]
    pub directional_inscattering_color_intensity: f32,
    #[br(if(is_dawntrail))]
    pub directional_inscattering_exponent: f32,
    #[br(if(is_dawntrail))]
    directional_inscattering_color_offset: i32,
    #[br(if(is_dawntrail))]
    #[brw(pad_after = 3)] // padding
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub use_height_fog_update: bool,

    #[br(if(is_dawntrail), seek_before = SeekFrom::Current(directional_inscattering_color_offset as i64 - Self::DAWNTRAIL_SIZE as i64), restore_position)]
    pub directional_inscattering_color: ColorIntensity,
}

impl VerticalFog {
    pub const SIZE: usize = 32;
    pub const DAWNTRAIL_SIZE: usize = Self::SIZE + 56;
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
