// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::{Cursor, Seek, SeekFrom};

use crate::ByteSpan;
use crate::common_file_operations::{read_bool_from, write_bool_as};
use binrw::{BinRead, BinReaderExt};
use binrw::{Endian, binrw};

mod aetheryte;
pub use aetheryte::AetheryteInstanceObject;

mod bg;
pub use bg::BGInstanceObject;
pub use bg::ModelCollisionType;

mod common;
pub use common::Color;
pub use common::ColorHDRI;
pub use common::Transformation;

mod env_set;
pub use env_set::EnvSetInstanceObject;
pub use env_set::EnvSetShape;

mod exit_range;
pub use exit_range::ExitRangeInstanceObject;
pub use exit_range::ExitType;

mod light;
pub use light::LightInstanceObject;
pub use light::LightType;
pub use light::PointLightType;

mod npc;
pub use npc::BNPCInstanceObject;
pub use npc::ENPCInstanceObject;
pub use npc::GameInstanceObject;
pub use npc::NPCInstanceObject;

mod pop;
pub use pop::PopRangeInstanceObject;
pub use pop::PopType;

mod position_marker;
pub use position_marker::PositionMarkerInstanceObject;
pub use position_marker::PositionMarkerType;

mod shared_group;
pub use shared_group::ColourState;
pub use shared_group::DoorState;
pub use shared_group::RotationState;
pub use shared_group::SharedGroupInstance;
pub use shared_group::TransformState;

mod sound;
pub use sound::SoundInstanceObject;

mod trigger_box;
pub use trigger_box::TriggerBoxInstanceObject;
pub use trigger_box::TriggerBoxShape;

mod string_heap;
pub use string_heap::{HeapPointer, HeapString, HeapStringFromPointer, StringHeap};

// From https://github.com/NotAdam/Lumina/tree/40dab50183eb7ddc28344378baccc2d63ae71d35/src/Lumina/Data/Parsing/Layer
// Also see https://github.com/aers/FFXIVClientStructs/blob/6b62122cae38bfbc016bf697bef75f80f37abac1/FFXIVClientStructs/FFXIV/Client/LayoutEngine/ILayoutInstance.cs

// TODO: convert these all to magic
#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum LayerEntryType {
    AssetNone = 00,
    BG = 0x1,
    Attribute = 0x2,
    LayLight = 0x3,
    Vfx = 0x4,
    PositionMarker = 0x5,
    SharedGroup = 0x6,
    Sound = 0x7,
    EventNPC = 0x8,
    BattleNPC = 0x9,
    RoutePath = 0xA,
    Character = 0xB,
    Aetheryte = 0xC,
    EnvSet = 0xD,
    Gathering = 0xE,
    HelperObject = 0xF,
    Treasure = 0x10,
    Clip = 0x11,
    ClipCtrlPoint = 0x12,
    ClipCamera = 0x13,
    ClipLight = 0x14,
    ClipReserve00 = 0x15,
    ClipReserve01 = 0x16,
    ClipReserve02 = 0x17,
    ClipReserve03 = 0x18,
    ClipReserve04 = 0x19,
    ClipReserve05 = 0x1A,
    ClipReserve06 = 0x1B,
    ClipReserve07 = 0x1C,
    ClipReserve08 = 0x1D,
    ClipReserve09 = 0x1E,
    ClipReserve10 = 0x1F,
    ClipReserve11 = 0x20,
    ClipReserve12 = 0x21,
    ClipReserve13 = 0x22,
    ClipReserve14 = 0x23,
    CutAssetOnlySelectable = 0x24,
    Player = 0x25,
    Monster = 0x26,
    Weapon = 0x27,
    PopRange = 0x28,
    /// Zone Transitions (the visible part is probably LineVFX?)
    ExitRange = 0x29,
    Lvb = 0x2A,
    MapRange = 0x2B,
    NaviMeshRange = 0x2C,
    EventObject = 0x2D,
    DemiHuman = 0x2E,
    EnvLocation = 0x2F,
    ControlPoint = 0x30,
    EventRange = 0x31,
    RestBonusRange = 0x32,
    QuestMarker = 0x33,
    Timeline = 0x34,
    ObjectBehaviorSet = 0x35,
    Movie = 0x36,
    ScenarioExd = 0x37,
    ScenarioText = 0x38,
    CollisionBox = 0x39,
    DoorRange = 0x3A,
    LineVFX = 0x3B,
    SoundEnvSet = 0x3C,
    CutActionTimeline = 0x3D,
    CharaScene = 0x3E,
    CutAction = 0x3F,
    EquipPreset = 0x40,
    ClientPath = 0x41,
    ServerPath = 0x42,
    GimmickRange = 0x43,
    TargetMarker = 0x44,
    ChairMarker = 0x45,
    ClickableRange = 0x46,
    PrefetchRange = 0x47,
    FateRange = 0x48,
    PartyMember = 0x49,
    KeepRange = 0x4A,
    SphereCastRange = 0x4B,
    IndoorObject = 0x4C,
    OutdoorObject = 0x4D,
    EditGroup = 0x4E,
    StableChocobo = 0x4F,
    MaxAssetType = 0x50,
    Unk1 = 90,
    Unk4 = 83, // seen in bg/ex5/01_xkt_x6/twn/x6t1/level/bg.lgb
    Unk2 = 86, // seen in bg/ex5/02_ykt_y6/fld/y6f1/level/bg.lgb
    Unk3 = 89, // seen in bg/ffxiv/sea_s1/fld/s1f3/level/planevent.lgb
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(magic: &LayerEntryType, string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub enum LayerEntryData {
    #[br(pre_assert(*magic == LayerEntryType::BG))]
    BG(#[brw(args(string_heap))] BGInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::LayLight))]
    LayLight(LightInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Vfx))]
    Vfx(VFXInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::PositionMarker))]
    PositionMarker(PositionMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::SharedGroup))]
    SharedGroup(#[brw(args(string_heap))] SharedGroupInstance),
    #[br(pre_assert(*magic == LayerEntryType::Sound))]
    Sound(SoundInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EventNPC))]
    EventNPC(ENPCInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::BattleNPC))]
    BattleNPC(BNPCInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Aetheryte))]
    Aetheryte(AetheryteInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EnvSet))]
    EnvSet(EnvSetInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Gathering))]
    Gathering(GatheringInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Treasure))]
    Treasure(TreasureInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::PopRange))]
    PopRange(PopRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::ExitRange))]
    ExitRange(ExitRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::MapRange))]
    MapRange(MapRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EventObject))]
    EventObject(EventInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EnvLocation))]
    EnvLocation(#[brw(args(string_heap))] EnvLocationObject),
    #[br(pre_assert(*magic == LayerEntryType::EventRange))]
    EventRange(EventRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::QuestMarker))]
    QuestMarker(QuestMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::CollisionBox))]
    CollisionBox(#[brw(args(string_heap))] CollisionBoxInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::LineVFX))]
    LineVFX(LineVFXInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::ClientPath))]
    ClientPath(ClientPathInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::ServerPath))]
    ServerPath(ServerPathInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::GimmickRange))]
    GimmickRange(GimmickRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::TargetMarker))]
    TargetMarker(TargetMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::ChairMarker))]
    ChairMarker(ChairMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::ClickableRange))]
    ClickableRange(ClickableRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::PrefetchRange))]
    PrefetchRange(PrefetchRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::FateRange))]
    FateRange(FateRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Unk1))]
    Unk1(),
    #[br(pre_assert(*magic == LayerEntryType::Unk2))]
    Unk2(),
    #[br(pre_assert(*magic == LayerEntryType::Unk3))]
    Unk3(),
    #[br(pre_assert(*magic == LayerEntryType::Unk4))]
    Unk4(),
    #[br(pre_assert(*magic == LayerEntryType::DoorRange))]
    DoorRange(),
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct VFXInstanceObject {
    pub asset_path_offset: u32,
    #[brw(pad_after = 4)] // padding
    pub soft_particle_fade_range: f32,
    pub color: Color,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 2)] // padding
    pub no_far_clip: bool,
    pub fade_near_start: f32,
    pub fade_near_end: f32,
    pub fade_far_start: f32,
    pub fade_far_end: f32,
    pub z_correct: f32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct GatheringInstanceObject {
    #[brw(pad_after = 4)] // padding
    pub gathering_point_id: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct TreasureInstanceObject {
    #[brw(pad_after = 11)] // padding
    pub nonpop_init_zone: u8,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct MapRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    map: u32,
    /// Name for the general location. Index into the PlaceName Sxcel sheet.
    pub place_name_block: u32,
    /// Name for the specific spot. Index into the PlaceName Sxcel sheet.
    pub place_name_spot: u32,
    weather: u32,
    bgm: u32,
    padding: [u8; 10],
    housing_block_id: u8,
    /// Most likely affects whether the EXP bonus affects this area.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub rest_bonus_effective: bool,
    /// Map discovery ID.
    pub discovery_id: u8,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    map_enabled: bool,
    /// Probably to enable indication in the little place name UI element.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub place_name_enabled: bool,
    /// Whether this place is discoverable (see `discovery_id`.)
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub discovery_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    bgm_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    weather_enabled: bool,
    /// Whether this area is marked as a sanctuary.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub rest_bonus_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    bgm_play_zone_in_only: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    lift_enabled: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    housing_enabled: bool,
    padding2: [u8; 2],
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct EventInstanceObject {
    pub parent_data: GameInstanceObject,
    /// A reference to another object, most likely.
    pub bound_instance_id: u32,
    #[brw(pad_after = 8)] // padding?
    pub linked_instance_id: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct EnvLocationObject {
    #[brw(args(string_heap))]
    pub ambient_light_asset_path: HeapString,
    #[brw(args(string_heap))]
    pub env_map_asset_path: HeapString,
    pub padding: [u8; 24], // TODO: UNKNOWN, MAYBE WRONG
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct EventRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub unk_flags: [u8; 12],
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct QuestMarkerInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct CollisionBoxInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    attribute_mask: u32,
    attribute: u32,
    push_player_out: u8,
    padding: [u8; 3],
    // TODO: this seems... wrong
    #[brw(args(string_heap))]
    collision_asset_path: HeapString,
    padding2: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct LineVFXInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct PathControlPoint {
    pub position: [f32; 3],
    pub point_id: u16,
    pub select: u8,
    pub _padding: u8,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct PathInstanceObject {
    pub control_points_unk: i32,
    #[br(temp)]
    #[bw(calc = control_points.len() as i32)]
    control_point_count: i32,
    _padding: [u32; 2],
    #[br(count = control_point_count)]
    pub control_points: Vec<PathControlPoint>,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ClientPathInstanceObject {
    pub parent_data: PathInstanceObject,
    pub ring: u8,
    _padding: u8,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ServerPathInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct GimmickRangeInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct TargetMarkerInstanceObject {}

#[binrw]
#[brw(repr = u32)]
#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum ChairType {
    Chair = 0x0,
    Bed = 0x1,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ChairMarkerInstanceObject {
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    left_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    right_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    back_enable: bool,
    padding: u8,
    chair_type: ChairType,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ClickableRangeInstanceObject {}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct PrefetchRangeInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    pub bound_instance_id: u32,
    padding: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct FateRangeInstanceObject {}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum LayerSetReferencedType {
    All = 0x0,
    Include = 0x1,
    Exclude = 0x2,
    Undetermined = 0x3,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(data_heap: &StringHeap, string_heap: &StringHeap), stream = r)]
#[bw(import(data_heap: &mut StringHeap, string_heap: &mut StringHeap))]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct LayerHeader {
    pub layer_id: u32,

    #[brw(args(string_heap))]
    pub name: HeapString,

    pub instance_object_offset: i32,
    pub instance_object_count: i32,

    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub tool_mode_visible: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub tool_mode_read_only: bool,

    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_bush_layer: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub ps3_visible: bool,
    #[br(temp)]
    #[bw(calc = data_heap.get_free_offset_args(&layer_set_referenced_list))]
    pub layer_set_referenced_list_offset: i32,
    #[br(calc = data_heap.read_args(r, layer_set_referenced_list_offset))]
    #[bw(ignore)]
    pub layer_set_referenced_list: LayerSetReferencedList,
    pub festival_id: u16,
    pub festival_phase_id: u16,
    pub is_temporary: u8,
    pub is_housing: u8,
    pub version_mask: u16,

    #[brw(pad_before = 4)]
    pub ob_set_referenced_list: i32,
    pub ob_set_referenced_list_count: i32,
    pub ob_set_enable_referenced_list: i32,
    pub ob_set_enable_referenced_list_count: i32,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct LayerSetReferenced {
    pub layer_set_id: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(data_heap: &StringHeap), stream = r)]
#[bw(import(data_heap: &mut StringHeap))]
pub struct LayerSetReferencedList {
    pub(crate) referenced_type: LayerSetReferencedType,
    #[br(temp)]
    #[bw(calc = data_heap.get_free_offset(&layer_sets))]
    layer_set_offset: i32,
    #[bw(calc = layer_sets.len() as i32)]
    pub layer_set_count: i32,

    #[br(count = layer_set_count)]
    #[bw(ignore)]
    pub layer_sets: Vec<LayerSetReferenced>,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct OBSetReferenced {
    asset_type: LayerEntryType,
    instance_id: u32,
    ob_set_asset_path_offset: u32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct OBSetEnableReferenced {
    asset_type: LayerEntryType,
    instance_id: u32,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    ob_set_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    ob_set_emissive_enable: bool,
    padding: [u8; 2],
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct InstanceObject {
    asset_type: LayerEntryType,
    pub instance_id: u32,
    #[brw(args(string_heap))]
    pub name: HeapString,
    pub transform: Transformation,
    #[br(args(&asset_type, string_heap))]
    #[bw(args(string_heap))]
    pub data: LayerEntryData,
}

#[derive(Debug, PartialEq)]
pub struct Layer {
    pub header: LayerHeader,
    pub objects: Vec<InstanceObject>,
}

impl Layer {
    pub(crate) fn read(endianness: Endian, cursor: &mut Cursor<ByteSpan>) -> Option<Layer> {
        let old_pos = cursor.position();

        let string_heap = StringHeap::from(old_pos);
        let data_heap = StringHeap::from(old_pos);

        let header =
            LayerHeader::read_options(cursor, endianness, (&data_heap, &string_heap)).unwrap();

        let mut objects = Vec::new();
        // read instance objects
        {
            let mut instance_offsets = vec![0i32; header.instance_object_count as usize];
            for i in 0..header.instance_object_count {
                instance_offsets[i as usize] =
                    cursor.read_type_args::<i32>(endianness, ()).unwrap();
            }

            for i in 0..header.instance_object_count {
                cursor
                    .seek(SeekFrom::Start(
                        old_pos
                            + header.instance_object_offset as u64
                            + instance_offsets[i as usize] as u64,
                    ))
                    .unwrap();

                let start = cursor.stream_position().unwrap();
                let string_heap = StringHeap::from(start);

                objects
                    .push(InstanceObject::read_options(cursor, endianness, (&string_heap,)).ok()?);

                let after_immediate_read = cursor.stream_position().unwrap();

                let next_offset = if i + 1 < header.instance_object_count {
                    old_pos
                        + header.instance_object_offset as u64
                        + instance_offsets[i as usize + 1] as u64
                } else {
                    old_pos + header.ob_set_referenced_list as u64
                };

                let expected_size = next_offset as u64 - start;
                let actual_size = after_immediate_read - start;

                // TODO: remove this once all the objects are fixed!
                /*if cfg!(debug_assertions) && expected_size != actual_size {
                    println!(
                        "{:#?} doesn't match the expected size! it's supposed to be {} bytes, but we read {} instead",
                        objects.last(),
                        expected_size,
                        actual_size
                    );
                }*/
            }
        }

        // read ob set referenced
        {
            cursor
                .seek(SeekFrom::Start(
                    old_pos + header.ob_set_referenced_list as u64,
                ))
                .unwrap();
            for _ in 0..header.ob_set_referenced_list_count {
                OBSetReferenced::read_options(cursor, endianness, ()).unwrap();
            }
        }

        // read ob set enable referenced list
        {
            cursor
                .seek(SeekFrom::Start(
                    old_pos + header.ob_set_enable_referenced_list as u64,
                ))
                .unwrap();
            for _ in 0..header.ob_set_enable_referenced_list_count {
                OBSetEnableReferenced::read_options(cursor, endianness, ()).unwrap();
            }
        }

        Some(Layer { header, objects })
    }
}
