// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::{Cursor, Seek, SeekFrom};

use crate::ByteSpan;
use crate::common_file_operations::{read_bool_from, write_bool_as};
use crate::string_heap::{HeapString, StringHeap};
use binrw::{BinRead, BinReaderExt};
use binrw::{Endian, binrw};

mod aetheryte;
pub use aetheryte::AetheryteInstanceObject;

mod bg;
pub use bg::{BGInstanceObject, ModelCollisionType};

mod collision;
pub use collision::{CollisionBoxInstanceObject, TriggerBoxInstanceObject, TriggerBoxShape};

mod common;
pub use common::{Color, ColorHDRI, GameInstanceObject, RelativePositions};

mod env;
pub use env::{EnvLocationObject, EnvSetInstanceObject, EnvSetShape};

mod event;
pub use event::EventInstanceObject;

mod gathering;
pub use gathering::GatheringInstanceObject;

mod light;
pub use light::{LightInstanceObject, LightType, PointLightType};

mod marker;
pub use marker::{
    ChairMarkerInstanceObject, ChairType, PositionMarkerInstanceObject, PositionMarkerType,
    QuestMarkerInstanceObject, TargetMarkerInstanceObject,
};

mod npc;
pub use npc::{BNPCInstanceObject, ENPCInstanceObject, NPCInstanceObject};

mod path;
pub use path::{
    ClientPathInstanceObject, PathControlPoint, PathInstanceObject, ServerPathInstanceObject,
};

mod range;
pub use range::{
    ClickableRangeInstanceObject, EventRangeInstanceObject, ExitRangeInstanceObject, ExitType,
    FateRangeInstanceObject, GimmickRangeInstanceObject, MapRangeInstanceObject,
    PopRangeInstanceObject, PopType, PrefetchRangeInstanceObject,
};

mod shared_group;
pub use shared_group::{
    ColourState, DoorState, RotationState, SharedGroupInstance, TransformState,
};

mod sound;
pub use sound::SoundInstanceObject;

mod treasure;
pub use treasure::TreasureInstanceObject;

mod vfx;
pub use vfx::{LineVFXInstanceObject, VFXInstanceObject};

// From https://github.com/NotAdam/Lumina/tree/40dab50183eb7ddc28344378baccc2d63ae71d35/src/Lumina/Data/Parsing/Layer
// Also see https://github.com/aers/FFXIVClientStructs/blob/6b62122cae38bfbc016bf697bef75f80f37abac1/FFXIVClientStructs/FFXIV/Client/LayoutEngine/ILayoutInstance.cs

// TODO: convert these all to magic
#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum LayerEntryType {
    None = 0x0,
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
    /// Representing nothing.
    #[br(pre_assert(*magic == LayerEntryType::None))]
    None,
    /// Background model.
    #[br(pre_assert(*magic == LayerEntryType::BG))]
    BG(#[brw(args(string_heap))] BGInstanceObject),
    /// Light source.
    #[br(pre_assert(*magic == LayerEntryType::LayLight))]
    LayLight(LightInstanceObject),
    /// Visual effect.
    #[br(pre_assert(*magic == LayerEntryType::Vfx))]
    Vfx(VFXInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::PositionMarker))]
    PositionMarker(PositionMarkerInstanceObject),
    /// Instance of a prefab.
    #[br(pre_assert(*magic == LayerEntryType::SharedGroup))]
    SharedGroup(#[brw(args(string_heap))] SharedGroupInstance),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Sound))]
    Sound(SoundInstanceObject),
    /// Event NPC.
    #[br(pre_assert(*magic == LayerEntryType::EventNPC))]
    EventNPC(ENPCInstanceObject),
    /// Battle NPC.
    #[br(pre_assert(*magic == LayerEntryType::BattleNPC))]
    BattleNPC(BNPCInstanceObject),
    /// Aetheryte.
    #[br(pre_assert(*magic == LayerEntryType::Aetheryte))]
    Aetheryte(AetheryteInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::EnvSet))]
    EnvSet(EnvSetInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Gathering))]
    Gathering(GatheringInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Treasure))]
    Treasure(TreasureInstanceObject),
    /// Used for a variety of things, including teleport locations.
    #[br(pre_assert(*magic == LayerEntryType::PopRange))]
    PopRange(PopRangeInstanceObject),
    /// Walkable transitions between zones.
    #[br(pre_assert(*magic == LayerEntryType::ExitRange))]
    ExitRange(ExitRangeInstanceObject),
    /// Locations on the map, such as sanctuaries.
    #[br(pre_assert(*magic == LayerEntryType::MapRange))]
    MapRange(MapRangeInstanceObject),
    /// Event object.
    #[br(pre_assert(*magic == LayerEntryType::EventObject))]
    EventObject(EventInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::EnvLocation))]
    EnvLocation(#[brw(args(string_heap))] EnvLocationObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::EventRange))]
    EventRange(EventRangeInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::QuestMarker))]
    QuestMarker(QuestMarkerInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::CollisionBox))]
    CollisionBox(#[brw(args(string_heap))] CollisionBoxInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::LineVFX))]
    LineVFX(LineVFXInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::ClientPath))]
    ClientPath(ClientPathInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::ServerPath))]
    ServerPath(ServerPathInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::GimmickRange))]
    GimmickRange(GimmickRangeInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::TargetMarker))]
    TargetMarker(TargetMarkerInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::ChairMarker))]
    ChairMarker(ChairMarkerInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::ClickableRange))]
    ClickableRange(ClickableRangeInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::PrefetchRange))]
    PrefetchRange(PrefetchRangeInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::FateRange))]
    FateRange(FateRangeInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Unk1))]
    Unk1(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Unk2))]
    Unk2(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Unk3))]
    Unk3(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Unk4))]
    Unk4(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::DoorRange))]
    DoorRange(),
    /// Unhandled or unknown type.
    Unknown(),
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum LayerSetReferencedType {
    All = 0x0,
    Include = 0x1,
    Exclude = 0x2,
    Undetermined = 0x3,
}

/// Metadata information for a [Layer].
#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(data_heap: &StringHeap, string_heap: &StringHeap), stream = r)]
#[bw(import(data_heap: &mut StringHeap, string_heap: &mut StringHeap))]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct LayerHeader {
    /// ID of this layer.
    pub layer_id: u32,

    /// The name of this layer.
    #[brw(args(string_heap))]
    pub name: HeapString,

    pub(crate) instance_object_offset: i32,
    pub(crate) instance_object_count: i32,

    /// Whether this layer is only visible in tool mode.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub tool_mode_visible: bool,
    /// Whether this layer is supposed to be read-only in tool mode.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub tool_mode_read_only: bool,

    /// Whether this is a bush layer.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_bush_layer: bool,

    /// If this layer should be visible on the Playstation 3.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub ps3_visible: bool,

    #[br(temp)]
    #[bw(calc = data_heap.get_free_offset_args(&layer_set_referenced_list))]
    pub(crate) layer_set_referenced_list_offset: i32,

    /// The other referenced layer sets.
    #[br(calc = data_heap.read_args(r, layer_set_referenced_list_offset))]
    #[bw(ignore)]
    pub layer_set_referenced_list: LayerSetReferencedList,
    /// The festival ID associated with this layer.
    pub festival_id: u16,
    /// The festival phase ID associated with this layer.
    pub festival_phase_id: u16,
    pub(crate) is_temporary: u8,
    pub(crate) is_housing: u8,
    pub(crate) version_mask: u16,

    #[brw(pad_before = 4)]
    pub(crate) ob_set_referenced_list: i32,
    pub(crate) ob_set_referenced_list_count: i32,
    pub(crate) ob_set_enable_referenced_list: i32,
    pub(crate) ob_set_enable_referenced_list_count: i32,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct LayerSetReferenced {
    /// The ID of the referenced layer set.
    pub layer_set_id: u32,
}

#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(data_heap: &StringHeap), stream = r)]
#[bw(import(data_heap: &mut StringHeap))]
pub struct LayerSetReferencedList {
    /// The tpye of reference.
    pub referenced_type: LayerSetReferencedType,
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

/// Transformation within the world space.
#[binrw]
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct Transformation {
    /// X, Y, Z of the location in space.
    pub translation: [f32; 3],
    /// Yaw, pitch and roll of the rotation in space.
    pub rotation: [f32; 3],
    /// Width, height and depth of the scale in space.
    pub scale: [f32; 3],
}

/// Represents a single object in a [Layer], which could be anything from a light to an aetheryte.
#[binrw]
#[derive(Debug, PartialEq)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct InstanceObject {
    asset_type: LayerEntryType,
    /// The unique ID of this object.
    pub instance_id: u32,
    /// The name of this object.
    #[brw(args(string_heap))]
    pub name: HeapString,
    /// The object's transformation in space.
    pub transform: Transformation,
    /// The data associated with this object.
    #[br(args(&asset_type, string_heap))]
    #[bw(args(string_heap))]
    pub data: LayerEntryData,
}

/// Represents a layer of [InstanceObject].
#[derive(Debug, PartialEq)]
pub struct Layer {
    /// The header for this layer.
    pub header: LayerHeader,
    /// The list of objects contained within this layer.
    pub objects: Vec<InstanceObject>,
}

impl Layer {
    /// Read from `cursor` with `endianness`.
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
                // TODO: check if we hit unknown/unhandled data types too
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
