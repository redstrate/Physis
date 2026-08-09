// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::{BinResult, BinWrite, binrw};

use crate::{
    common_file_operations::{
        read_bool_from, read_dawntrail_marker, read_null_terminated_utf8, write_bool_as,
        write_dawntrail_marker,
    },
    layer::Layer,
    string_heap::{HeapPointer, HeapString, StringHeap},
    tmb::Tmb,
};

#[binrw::writer(writer, endian)]
pub(crate) fn write_scns(scns: &Vec<ScnSection>, string_heap: &mut StringHeap) -> BinResult<()> {
    for scn in scns {
        scn.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[derive(Debug, Default)]
pub struct ScnLayerGroup {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    pub layer_group_id: u32,

    #[brw(args(heap_pointer, string_heap))]
    pub name: HeapString,

    layer_offsets_start: i32,
    layer_offsets_count: i32,

    #[br(count = layer_offsets_count)]
    #[br(seek_before = SeekFrom::Current(layer_offsets_start as i64 - ScnLayerGroup::SIZE as i64))]
    #[br(restore_position)]
    offsets_layers: Vec<i32>,

    #[br(restore_position, parse_with = layers_from_offsets, args(&offsets_layers, string_heap))]
    #[br(seek_before = SeekFrom::Current(layer_offsets_start as i64 - ScnLayerGroup::SIZE as i64))]
    #[bw(ignore)] // TODO: support writing
    pub layers: Vec<Layer>,
}

impl ScnLayerGroup {
    pub(crate) const SIZE: usize = 0x10;
}

/// SCN1 section used in LVBs and SGBs.
#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
#[brw(magic = b"SCN1")]
pub struct ScnSection {
    /// Size of this header. Should be equal to `ScnHeader::SIZE`.
    total_size: u32,
    /// Offset to FileLayerGroupHeader[NumEmbeddedLayerGroups].
    pub(crate) offset_layer_groups: i32,
    /// Number of embedded layer groups.
    pub(crate) num_layer_groups: i32,
    /// Offset to FileSceneGeneral.
    offset_general: i32,
    /// Offset to ScnLayerSetsSection.
    offset_layer_sets: i32,
    /// Offset to FileSceneTimelineList.
    offset_timelines: i32,
    /// offset to a list of path offsets (ints)
    offset_layer_group_resources: i32,
    num_layer_group_resources: i32,
    unk2: i32,
    offset_action_descriptors: i32,
    unk4: i32,
    unk5: i32,
    offset_stain_info: i32,
    unk7: i32,
    unk8: i32,
    unk9: i32,
    unk10: i32,

    /// List of embedded layer groups.
    #[br(count = num_layer_groups, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset_layer_groups as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub layer_groups: Vec<ScnLayerGroup>,

    /// General information.
    #[br(seek_before = SeekFrom::Current(offset_general as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    #[brw(args(string_heap))]
    pub general: ScnGeneralSection,

    /// Layer set information.
    #[br(seek_before = SeekFrom::Current(offset_layer_sets as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    #[brw(args(string_heap))]
    pub layer_sets: ScnLayerSetsSection,

    /// Embedded animation timelines.
    #[br(seek_before = SeekFrom::Current(offset_timelines as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    #[br(args(string_heap))]
    pub timelines: ScnTimelinesSection,

    #[br(count = num_layer_group_resources)]
    #[br(seek_before = SeekFrom::Current(offset_layer_group_resources as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    offset_path_layer_group_resources: Vec<i32>,

    /// Associated [crate::lgb] paths.
    #[br(parse_with = strings_from_offsets)]
    #[br(args(&offset_path_layer_group_resources))]
    #[br(restore_position)]
    #[br(seek_before = SeekFrom::Current(offset_layer_group_resources as i64 - ScnSection::SIZE as i64))]
    #[bw(ignore)] // TODO: support
    pub lgb_paths: Vec<String>,

    /// Animation action descriptors.
    #[br(seek_before = SeekFrom::Current(offset_action_descriptors as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    pub action_descriptors: ScnSGActionDescriptors,

    /// Stain information, mainly used for housing items.
    #[br(if(offset_stain_info > 0), seek_before = SeekFrom::Current(offset_stain_info as i64 - ScnSection::SIZE as i64))]
    #[br(restore_position)]
    pub stain_info: Option<ScnStainInformation>,
}

impl ScnSection {
    pub(crate) const SIZE: usize = 0x40;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[derive(Debug)]
pub struct ScnEnvSpace {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    /// Path to an `.envb` file.
    #[brw(args(heap_pointer, string_heap))]
    pub envb_path: HeapString,

    /// Index into EnvScene.EnvSpaces.
    pub index: i32,

    /// ID to an EnvLocation InstanceObject in this scene.
    pub env_location_instance_id: i32,

    /// Path to a `.essb` file.
    #[brw(args(heap_pointer, string_heap))]
    pub essb_path: HeapString,

    unk1: f32,
    unk2: f32,
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[derive(Debug, Default)]
pub struct ScnGeneralSection {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    flags1: [u8; 4],

    #[brw(args(heap_pointer, string_heap))]
    pub bg_path: HeapString,

    offset_env_spaces: i32,
    #[br(temp)]
    #[bw(calc = env_spaces.len() as i32)]
    num_env_spaces: i32,

    /// Int casted to float. Used by LayoutEnvironment.
    unk1: i32,

    /// Path to the `.svb` file.
    #[brw(args(heap_pointer, string_heap))]
    pub svb_path: HeapString,

    // All these floats are also environmental data!
    unk2: f32,
    unk3: f32,
    unk4: f32,
    unk5: f32,
    unk6: f32,
    unk7: f32,
    unk8_offset: i32,

    /// Path to the `.lcb` file.
    #[brw(args(heap_pointer, string_heap))]
    pub lcb_path: HeapString,

    flags2: [u8; 4],
    unk11: f32,
    weather_ids_offset: i32,
    unk13: f32,
    unk14: f32,
    flags3: [u8; 4],
    unk16: f32,
    /// Only read if `is_dawntrail` is true.
    unk17: f32,
    #[br(map = read_dawntrail_marker)]
    #[bw(map = write_dawntrail_marker)]
    pub is_dawntrail: bool,

    #[br(count = num_env_spaces)]
    #[br(seek_before = SeekFrom::Current(offset_env_spaces as i64 - ScnGeneralSection::SIZE as i64))]
    #[br(restore_position)]
    #[br(args { inner: (string_heap,) })]
    #[bw(write_with = write_env_spaces, args(string_heap))]
    pub env_spaces: Vec<ScnEnvSpace>,

    #[br(count = 32)]
    #[br(seek_before = SeekFrom::Current(weather_ids_offset as i64 - ScnGeneralSection::SIZE as i64 + 4))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO
    pub weather_ids: Vec<u8>,

    /// I think these are all casted to float by dividing by 255.0?
    #[br(seek_before = SeekFrom::Current(unk8_offset as i64 - ScnGeneralSection::SIZE as i64 + 4))]
    #[br(try, restore_position)] // TODO: if try isn't here, SGBs like bg/ffxiv/sea_s1/shared/for_bg/sgbg_s1d1_m1_sec1.sgb fail?
    #[bw(ignore)] // TODO
    unk8a: [u8; 3],
}

#[binrw::writer(writer, endian)]
pub fn write_env_spaces(scns: &Vec<ScnEnvSpace>, string_heap: &mut StringHeap) -> BinResult<()> {
    for scn in scns {
        scn.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

impl ScnGeneralSection {
    pub(crate) const SIZE: usize = 0x5C;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[derive(Debug, Default)]
pub struct ScnTimelinesSection {
    offset_entries: i32,
    num_entries: i32,

    #[br(seek_before = SeekFrom::Current(offset_entries as i64 - ScnTimelinesSection::SIZE as i64))]
    #[br(count = num_entries, restore_position, args { inner: (string_heap,) })]
    #[bw(ignore)] // TODO: support writing
    pub timelines: Vec<ScnTimeline>,
}

impl ScnTimelinesSection {
    pub(crate) const SIZE: usize = 0x8;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[derive(Debug, Default)]
pub struct ScnTimeline {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    pub sub_id: u32,
    #[brw(args(heap_pointer, string_heap))]
    pub animation_type: HeapString,
    offset_instances: i32,
    num_instances: i32,
    offset_action_timeline_key: i32, // TODO: may be be a string? or at least clientstructs claims its one
    offset_tmb: i32,
    tmb_size: i32,

    unk1: [u8; 4], // empty(?)
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub loop_animation: bool,
    unk2: [u8; 10], // unsure

    /// Bytes of a TMLB file.
    #[br(seek_before = SeekFrom::Current(offset_tmb as i64 - ScnTimeline::SIZE as i64), restore_position)]
    #[brw(args(string_heap,))]
    pub tmb: Tmb,

    #[br(seek_before = SeekFrom::Current(offset_instances as i64 - ScnTimeline::SIZE as i64))]
    #[br(count = num_instances, restore_position)]
    pub instances: Vec<ScnTimelineInstance>,
}

impl ScnTimeline {
    pub(crate) const SIZE: usize = 0x2C;
}

#[binrw]
#[derive(Debug, Clone)]
#[repr(C)]
pub struct ScnTimelineInstance {
    /// Points to a [crate::tmb::Tmac] node with this `time`.
    pub tmac_time: i32,
    /// Points to an instance object ID in the embedded layer groups.
    pub instance_id: i32,
}

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, Clone, Copy, Default)]
pub enum SGStateMode {
    #[default]
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2, // This isn't used by the client?
    OnOff = 3,
    Unk4 = 4,
}

#[binrw]
#[derive(Debug, Default)]
pub struct ScnSGActionDescriptors {
    unk1: u8,
    /// Sub ID of the timeline for the "on" state. Only read when using `SGStateMode::OnOff`.
    pub on_sub_id: u8, // read when unk7 is 3
    /// Sub ID of the timeline for the "off" state. Only read when using `SGStateMode::OnOff`.
    pub off_sub_id: u8,
    unk4: [u8; 2],
    /// List of timeline sub IDs which can be played and stopped in-game.
    pub timeline_indices: [u8; 16],
    unk5: [u8; 2],
    unk6_bool: u8, // initializes something
    pub state_mode: SGStateMode,
    /// Only read when using `SGStateMode::Unk4`.
    unk8: u16,
    /// Only read when using `SGStateMode::Unk4`.
    unk9: u16,
    unk10: [u8; 32],
    #[br(temp)]
    #[bw(calc = descriptors.len() as i32)]
    count: i32,
    unk11: [u8; 4],
    #[br(count = count, try)]
    // TODO: Remove the try once we know why count for bg/ffxiv/sea_s1/shared/for_bg/sgbg_s1d1_m1_dorb1.sgb is wrong.
    pub descriptors: Vec<ScnSGActionControllerDescriptor>,
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone)]
pub enum ScnSGActionControllerDescriptor {
    #[brw(magic = 1i32)]
    Door(ScnDoorActionDescription),
    #[brw(magic = 2i32)]
    Rotation(ScnRotationActionDescription),
    // TODO: 3 = TransformAction
    // TODO: 4 = ClockAction
    // TODO: 5 = TransformAction again(?)
    // TODO: 6 = TransformAction again(?!)
    Unknown(i32),
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ScnDoorActionDescription {
    check_if_1: u8, // client checks if this isn't 0, but im unsure what is after this
    unk: [u8; 11],  // seems to be empty, but not confirmed yet
    /// Instance ID of a BG object to animate.
    pub door_object_0: u8,
    /// Instance ID of a BG object to animate.
    pub door_object_1: u8,
    unk2: [u8; 2],
    pub door_type: i32,
    unk1: f32,
    pub max_rotation: f32,
    pub max_translation: f32,
    /// Instance ID of a sound to play.
    pub sound_0: u8,
    /// Instance ID of a sound to play.
    pub sound_1: u8,
    /// Instance ID of a BG object to animate.
    pub door_object_2: u8,
    /// Instance ID of a BG object to animate.
    pub door_object_3: u8,
    // TODO: unsure if there's other things here
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum RotationAxis {
    #[brw(magic = 0i32)]
    X,
    #[brw(magic = 1i32)]
    Y,
    #[brw(magic = 2i32)]
    Z,
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct ScnRotationActionDescription {
    check_if_1: u8, // client checks if this isn't 0, but im unsure what is after this
    unk: [u8; 11],  // seems to be empty, but not confirmed yet
    /// Instance ID of the BG object to animate.
    pub bg_part_id: u8,
    unk2: [u8; 3], // seems to be empty, but not confirmed yet
    pub axis: RotationAxis,
    // TODO: figure out the units
    pub duration: f32,
    /// How many degrees to add during the `duration`.
    pub value: f32,
    pub vfx_child1_id: u8,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub vfx_has_child1: bool,
    unk_fields: [u8; 4], // read individually as bytes, but i don't know what they do,
    pub vfx_child_2_id: u8,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub vfx_has_child2: bool,
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, Default)]
pub struct ScnLayerSetsSection {
    offset: i32,
    count: i32,

    #[br(seek_before = SeekFrom::Current(offset as i64 - ScnLayerSetsSection::SIZE as i64))]
    #[br(count = count, restore_position, args { inner: (string_heap,) })]
    #[bw(write_with = write_layersets, args(string_heap))]
    pub layer_sets: Vec<ScnLayerSet>,
}

#[binrw::writer(writer, endian)]
pub fn write_layersets(scns: &Vec<ScnLayerSet>, string_heap: &mut StringHeap) -> BinResult<()> {
    for scn in scns {
        scn.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

impl ScnLayerSetsSection {
    pub(crate) const SIZE: usize = 0x8;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[derive(Debug)]
pub struct ScnLayerSet {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    /// Path to the `.nvm` file for this layer set.
    #[brw(args(heap_pointer, string_heap))]
    pub nvm_path: HeapString,

    /// The ID of this layer set.
    pub id: i32,
    unk2: i32,
    unk3: i32,

    /// Refers to a row in the TerritoryType Excel sheet.
    pub territory_type_id: u16,
    /// Refers to a row in the ContentFinderCondition Excel sheet.
    pub content_finder_condition_id: u16,

    unk5: i32,

    /// Path to the `.nvx` file for this layer set.
    #[brw(args(heap_pointer, string_heap))]
    pub nvx_path: HeapString,
}

#[binrw::parser(reader)]
fn strings_from_offsets(offsets: &Vec<i32>) -> BinResult<Vec<String>> {
    let base_offset = reader.stream_position()?;

    let mut strings: Vec<String> = vec![];

    for offset in offsets {
        let string_offset = *offset as u64;

        reader.seek(SeekFrom::Start(base_offset + string_offset))?;
        strings.push(read_null_terminated_utf8(reader));
    }

    Ok(strings)
}

#[binrw::parser(reader, endian)]
fn layers_from_offsets(offsets: &Vec<i32>, string_heap: &StringHeap) -> BinResult<Vec<Layer>> {
    let base_offset = reader.stream_position()?;

    let mut layers: Vec<Layer> = vec![];

    for offset in offsets {
        let layer_offset = *offset as u64;

        reader.seek(SeekFrom::Start(base_offset + layer_offset))?;
        // TODO: need separate data heap eventually
        layers.push(Layer::read(endian, reader, string_heap, string_heap)?);
    }

    Ok(layers)
}

#[binrw]
#[derive(Debug, Default)]
pub struct ScnStainInformation {
    pub default_stain_index: u16,
    unk1: u8,
    unk2: u8,
    unk3: u32,
    unk4: [u32; 6],
    unk5: u32,
    unk6: u32,
    unk7: u32,
    unk8: u32,
    unk9: u32,
    unk10: u8,
    unk11: [u8; 3],
    unk12: u32,
    unk13: u32,
    unk14: u32,
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_scnlayergroup_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<ScnLayerGroup, { ScnLayerGroup::SIZE }>();
    }

    #[test]
    fn test_scnsection_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<ScnSection, { ScnSection::SIZE }>();
    }

    #[test]
    fn test_scngeneralsection_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<ScnGeneralSection, { ScnGeneralSection::SIZE }>();
    }

    #[test]
    fn test_scntimelinessection_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<ScnTimelinesSection, { ScnTimelinesSection::SIZE }>();
    }

    #[test]
    fn test_scntimeline_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<ScnTimeline, { ScnTimeline::SIZE }>();
    }

    #[test]
    fn test_scnlayersetssection_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<ScnLayerSetsSection, { ScnLayerSetsSection::SIZE }>();
    }
}
