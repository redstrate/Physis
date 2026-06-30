// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::{Read, Seek, SeekFrom};

use crate::common_file_operations::{read_bool_from, write_bool_as};
use crate::string_heap::{HeapPointer, HeapString, StringHeap};
use binrw::{BinRead, BinReaderExt, BinResult};
use binrw::{Endian, binrw};

mod aetheryte;
pub use aetheryte::AetheryteInstanceObject;

mod bg;
pub use bg::{BGInstanceObject, ModelCollisionType};

mod collision;
pub use collision::{CollisionBoxInstanceObject, TriggerBoxInstanceObject, TriggerBoxShape};

mod common;
pub use common::{Color, ColorHDRI, GameInstanceObject};

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
    QuestMarkerInstanceObject, TargetMarkerInstanceObject, TargetMarkerType,
};

mod npc;
pub use npc::{BNPCInstanceObject, ENPCInstanceObject, NPCInstanceObject};

mod path;
pub use path::{
    ClientPathInstanceObject, PathControlPoint, PathInstanceObject, ServerPathInstanceObject,
};

mod range;
pub use range::{
    ClickableRangeInstanceObject, DoorRangeInstanceObject, EventRangeInstanceObject,
    ExitRangeInstanceObject, ExitType, FateRangeInstanceObject, GimmickRangeInstanceObject,
    MapRangeInstanceObject, PopRangeInstanceObject, PopType, PrefetchRangeInstanceObject,
};

mod shared_group;
pub use shared_group::{
    ColourState, DoorState, RotationState, SharedGroupInstance, TransformState,
};

mod sound;
pub use sound::{SoundEffectType, SoundInstanceObject, SoundParameters};

mod transformation;
pub use transformation::Transformation;

mod treasure;
pub use treasure::TreasureInstanceObject;

mod vfx;
pub use vfx::{LineStyle, LineVFXInstanceObject, VFXInstanceObject};

// From https://github.com/NotAdam/Lumina/tree/40dab50183eb7ddc28344378baccc2d63ae71d35/src/Lumina/Data/Parsing/Layer
// Also see https://github.com/aers/FFXIVClientStructs/blob/6b62122cae38bfbc016bf697bef75f80f37abac1/FFXIVClientStructs/FFXIV/Client/LayoutEngine/ILayoutInstance.cs

// TODO: convert these all to magic
#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum LayerEntryType {
    /// This represents nothing.
    None = 0,
    /// Background model object.
    BG = 1,
    Attribute = 2,
    /// Light object.
    LayLight = 3,
    /// Visual effect object.
    Vfx = 4,
    PositionMarker = 5,
    /// Shared group object.
    SharedGroup = 6,
    /// Sound object.
    Sound = 7,
    /// Event NPC object.
    EventNPC = 8,
    /// Battle NPC object. These are stripped out of the released client.
    BattleNPC = 9,
    RoutePath = 10,
    Character = 11,
    /// Aetheryte object.
    Aetheryte = 12,
    EnvSet = 13,
    /// Gathering point. These are stripped out of the released client.
    Gathering = 14,
    HelperObject = 15,
    /// Treasure object.
    Treasure = 16,
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
    /// Generic range for characters to spawn in.
    PopRange = 40,
    /// Zone Transitions (the visible part is probably LineVFX?)
    ExitRange = 41,
    Lvb = 42,
    MapRange = 0x2B,
    NaviMeshRange = 0x2C,
    /// Event object.
    EventObject = 45,
    DemiHuman = 0x2E,
    EnvLocation = 0x2F,
    ControlPoint = 0x30,
    /// Generic ranges for events to use.
    EventRange = 49,
    RestBonusRange = 0x32,
    QuestMarker = 0x33,
    Timeline = 0x34,
    ObjectBehaviorSet = 0x35,
    Movie = 0x36,
    ScenarioExd = 0x37,
    ScenarioText = 0x38,
    CollisionBox = 0x39,
    DoorRange = 0x3A,
    /// Generic VFX that displays those dotted lines used for zone transitions and boundaries.
    LineVFX = 0x3B,
    SoundEnvSet = 0x3C,
    CutActionTimeline = 0x3D,
    CharaScene = 0x3E,
    CutAction = 0x3F,
    EquipPreset = 0x40,
    /// Path object.
    ClientPath = 65,
    /// Path object that (presumably) only exists on the server.
    ServerPath = 66,
    GimmickRange = 67,
    TargetMarker = 68,
    /// Place for a character to sit.
    ChairMarker = 69,
    ClickableRange = 70,
    PrefetchRange = 71,
    FateRange = 72,
    PartyMember = 73,
    KeepRange = 0x4A,
    SphereCastRange = 0x4B,
    IndoorObject = 0x4C,
    OutdoorObject = 0x4D,
    EditGroup = 0x4E,
    StableChocobo = 0x4F,
    Unk4 = 0x53, // seen in bg/ex5/01_xkt_x6/twn/x6t1/level/bg.lgb
    Unk2 = 0x56, // seen in bg/ex5/02_ykt_y6/fld/y6f1/level/bg.lgb
    Unk5 = 0x57, // seen in bg/ex2/05_zon_z3/rad/z3r3/level/planmap.lgb
    Unk3 = 0x59, // seen in bg/ffxiv/sea_s1/fld/s1f3/level/planevent.lgb
    Unk1 = 0x5A, // Culling related?
    Unk6 = 0x5B, // Seen in disassembly
    Unk7 = 0x5C, // Ditto
    Unk8 = 0x5D, // Ditto x2
}

impl From<&LayerEntryData> for LayerEntryType {
    fn from(value: &LayerEntryData) -> Self {
        match value {
            LayerEntryData::None => LayerEntryType::None,
            LayerEntryData::BG(_) => LayerEntryType::BG,
            LayerEntryData::LayLight(_) => LayerEntryType::LayLight,
            LayerEntryData::Vfx(_) => LayerEntryType::Vfx,
            LayerEntryData::PositionMarker(_) => LayerEntryType::PositionMarker,
            LayerEntryData::SharedGroup(_) => LayerEntryType::SharedGroup,
            LayerEntryData::Sound(_) => LayerEntryType::Sound,
            LayerEntryData::EventNPC(_) => LayerEntryType::EventNPC,
            LayerEntryData::BattleNPC(_) => LayerEntryType::BattleNPC,
            LayerEntryData::Aetheryte(_) => LayerEntryType::Aetheryte,
            LayerEntryData::EnvSet(_) => LayerEntryType::EnvSet,
            LayerEntryData::Gathering(_) => LayerEntryType::Gathering,
            LayerEntryData::Treasure(_) => LayerEntryType::Treasure,
            LayerEntryData::PopRange(_) => LayerEntryType::PopRange,
            LayerEntryData::ExitRange(_) => LayerEntryType::ExitRange,
            LayerEntryData::MapRange(_) => LayerEntryType::MapRange,
            LayerEntryData::EventObject(_) => LayerEntryType::EventObject,
            LayerEntryData::EnvLocation(_) => LayerEntryType::EnvLocation,
            LayerEntryData::EventRange(_) => LayerEntryType::EventRange,
            LayerEntryData::QuestMarker(_) => LayerEntryType::QuestMarker,
            LayerEntryData::CollisionBox(_) => LayerEntryType::CollisionBox,
            LayerEntryData::LineVFX(_) => LayerEntryType::LineVFX,
            LayerEntryData::ClientPath(_) => LayerEntryType::ClientPath,
            LayerEntryData::ServerPath(_) => LayerEntryType::ServerPath,
            LayerEntryData::GimmickRange(_) => LayerEntryType::GimmickRange,
            LayerEntryData::TargetMarker(_) => LayerEntryType::TargetMarker,
            LayerEntryData::ChairMarker(_) => LayerEntryType::ChairMarker,
            LayerEntryData::ClickableRange(_) => LayerEntryType::ClickableRange,
            LayerEntryData::PrefetchRange(_) => LayerEntryType::PrefetchRange,
            LayerEntryData::FateRange(_) => LayerEntryType::FateRange,
            LayerEntryData::Unk1() => LayerEntryType::Unk1,
            LayerEntryData::Unk2() => LayerEntryType::Unk2,
            LayerEntryData::Unk3() => LayerEntryType::Unk3,
            LayerEntryData::Unk4() => LayerEntryType::Unk4,
            LayerEntryData::Unk5() => LayerEntryType::Unk5,
            LayerEntryData::Unk6() => LayerEntryType::Unk6,
            LayerEntryData::Unk7() => LayerEntryType::Unk7,
            LayerEntryData::Unk8() => LayerEntryType::Unk8,
            LayerEntryData::DoorRange(_) => LayerEntryType::DoorRange,
            LayerEntryData::Unknown() => unreachable!(),
        }
    }
}

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[br(import(magic: &LayerEntryType, string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub enum LayerEntryData {
    /// Representing nothing.
    #[default]
    #[br(pre_assert(*magic == LayerEntryType::None))]
    None,
    /// Background model.
    #[br(pre_assert(*magic == LayerEntryType::BG))]
    BG(#[brw(args(string_heap, heap_pointer))] BGInstanceObject),
    /// Light source.
    #[br(pre_assert(*magic == LayerEntryType::LayLight))]
    LayLight(LightInstanceObject),
    /// Visual effect.
    #[br(pre_assert(*magic == LayerEntryType::Vfx))]
    Vfx(#[brw(args(string_heap, heap_pointer))] VFXInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::PositionMarker))]
    PositionMarker(PositionMarkerInstanceObject),
    /// Instance of a prefab.
    #[br(pre_assert(*magic == LayerEntryType::SharedGroup))]
    SharedGroup(#[brw(args(string_heap, heap_pointer))] SharedGroupInstance),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Sound))]
    Sound(#[brw(args(string_heap, heap_pointer))] SoundInstanceObject),
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
    EnvSet(#[brw(args(string_heap, heap_pointer))] EnvSetInstanceObject),
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
    EnvLocation(#[brw(args(string_heap, heap_pointer))] EnvLocationObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::EventRange))]
    EventRange(EventRangeInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::QuestMarker))]
    QuestMarker(QuestMarkerInstanceObject),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::CollisionBox))]
    CollisionBox(#[brw(args(string_heap, heap_pointer))] CollisionBoxInstanceObject),
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
    #[br(pre_assert(*magic == LayerEntryType::Unk5))]
    Unk5(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Unk6))]
    Unk6(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Unk7))]
    Unk7(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::Unk8))]
    Unk8(),
    /// Unknown purpose.
    #[br(pre_assert(*magic == LayerEntryType::DoorRange))]
    DoorRange(DoorRangeInstanceObject),
    /// Unhandled or unknown type.
    Unknown(),
}

#[binrw]
#[repr(u32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Default, Copy, Clone)]
pub enum LayerSetReferencedType {
    #[default]
    All = 0x0,
    Include = 0x1,
    Exclude = 0x2,
    Undetermined = 0x3,
}

/// Metadata information for a [Layer].
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[br(import(endianness: Endian, data_heap: &StringHeap, string_heap: &StringHeap), stream = r)]
#[bw(import(data_heap: &mut StringHeap, string_heap: &mut StringHeap), stream = w)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct LayerHeader {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    /// ID of this layer.
    pub layer_id: u32,

    /// The name of this layer.
    #[brw(args(heap_pointer, string_heap))]
    pub name: HeapString,

    // TODO: remove these from public API!!
    /// This field should be left at it's default. This will be removed in a future version.
    pub instance_object_offset: i32,
    /// This field should be left at it's default. This will be removed in a future version.
    pub instance_object_count: i32,

    /// Whether this layer is visible by default. If false, it does not show up in game.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub visible: bool,

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
    #[bw(calc = data_heap.get_free_offset_args(&layer_set_referenced_list).saturating_sub(heap_pointer.pos as i32) - 12)]
    // lol 12
    pub(crate) layer_set_referenced_list_offset: i32,

    /// The layer set referenced list.
    #[br(calc = data_heap.read_args(r, endianness, heap_pointer, layer_set_referenced_list_offset))]
    #[bw(ignore)] // Written above
    pub layer_set_referenced_list: LayerSetReferencedList,

    /// Only show this layer if this festival ID is active.
    pub festival_id: u16,
    /// Only show this layer if this festival phase ID is active..
    pub festival_phase_id: u16,

    /// Whether this layer is temporary.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_temporary: bool,

    /// Whether this is a housing-related layer.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub is_housing: bool,

    /// Unknown purpose, but probably version related.
    pub version_mask: u16,

    #[brw(pad_before = 4)]
    #[br(temp)]
    #[bw(calc = data_heap.get_free_vec_offset_args(object_set_referenced, string_heap).saturating_sub(heap_pointer.pos as i32) - 12)]
    // lol again
    ob_set_referenced_list_offset: i32,
    #[bw(calc = object_set_referenced.len() as i32)]
    #[br(temp)]
    ob_set_referenced_list_count: i32,

    /// The object set referenced.
    #[br(calc = data_heap.read_vec_args(r, endianness, string_heap, heap_pointer, ob_set_referenced_list_count as usize, ob_set_referenced_list_offset))]
    #[bw(ignore)] // Written above
    pub object_set_referenced: Vec<ObjectSetReferenced>,

    #[br(temp)]
    #[bw(calc = data_heap.get_free_vec_offset_args(object_set_enable_referenced, string_heap).saturating_sub(heap_pointer.pos as i32) - 12)]
    // yea keeps going
    ob_set_enable_referenced_list_offset: i32,
    #[bw(calc = object_set_enable_referenced.len() as i32)]
    #[br(temp)]
    ob_set_enable_referenced_list_count: i32,

    /// The object set enable referenced.
    #[br(calc = data_heap.read_vec_args(r, endianness, string_heap, heap_pointer, ob_set_enable_referenced_list_count as usize, ob_set_enable_referenced_list_offset))]
    #[bw(ignore)] // Written above
    pub object_set_enable_referenced: Vec<ObjectSetEnableReferenced>,
}

impl LayerHeader {
    pub const SIZE: usize = 0x34;

    /// Whether this layer set ID is included or excluded.
    pub fn has_layer_set(&self, id: u32) -> bool {
        match self.layer_set_referenced_list.referenced_type {
            LayerSetReferencedType::Include => {
                self.layer_set_referenced_list.layer_set_ids.contains(&id)
            }
            LayerSetReferencedType::Exclude => {
                !self.layer_set_referenced_list.layer_set_ids.contains(&id)
            }
            LayerSetReferencedType::All => true, // NOTE: This is based on the assumption seen in The Lavender Beds (340)'s pop range in LVD_Zone_01.
            _ => false,                          // Unsure how the other ones should be handled yet
        }
    }
}

impl Default for LayerHeader {
    fn default() -> Self {
        Self {
            layer_id: Default::default(),
            name: Default::default(),
            instance_object_offset: Default::default(),
            instance_object_count: Default::default(),
            visible: true,
            tool_mode_read_only: Default::default(),
            is_bush_layer: Default::default(),
            ps3_visible: Default::default(),
            layer_set_referenced_list: Default::default(),
            festival_id: Default::default(),
            festival_phase_id: Default::default(),
            is_temporary: Default::default(),
            is_housing: Default::default(),
            version_mask: Default::default(),
            object_set_referenced: Default::default(),
            object_set_enable_referenced: Default::default(),
        }
    }
}

#[binrw]
#[br(import(data_heap: &StringHeap), stream = r)]
#[bw(import(data_heap: &mut StringHeap))]
#[derive(Debug, PartialEq, Default, Clone)]
pub struct LayerSetReferencedList {
    /// The type of reference.
    pub referenced_type: LayerSetReferencedType,
    #[br(temp)]
    #[bw(calc = data_heap.get_free_offset(&layer_set_ids))]
    layer_set_offset: i32,
    #[bw(calc = layer_set_ids.len() as i32)]
    #[br(temp)]
    layer_set_count: i32,

    /// Corresponds to IDs of a [ScnLayerSet](crate::scn::ScnLayerSet).
    #[br(count = layer_set_count)]
    #[bw(ignore)] // Written above
    pub layer_set_ids: Vec<u32>,
}

#[binrw]
#[br(import(string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectSetReferenced {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    /// The type of InstanceObject of `instance_id`.
    pub asset_type: LayerEntryType,

    /// Instance ID referring to an object within this LGB.
    pub instance_id: u32,

    /// Path to an `.obsb` file.
    #[brw(args(heap_pointer, string_heap))]
    pub obsb_path: HeapString,
}

#[binrw]
#[br(import(string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug, PartialEq, Clone)]
pub struct ObjectSetEnableReferenced {
    pub asset_type: LayerEntryType,
    pub instance_id: u32,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub ob_set_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub ob_set_emissive_enable: bool,
    padding: [u8; 2],
}

/// Represents a single object in a [Layer], which could be anything from a light to an aetheryte.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct InstanceObject {
    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    #[bw(calc = data.into())]
    #[br(temp)]
    asset_type: LayerEntryType,
    /// The unique ID of this object.
    pub instance_id: u32,
    /// The name of this object.
    #[brw(args(heap_pointer, string_heap))]
    pub name: HeapString,
    /// The object's transformation in the world space.
    pub transform: Transformation,
    /// The data associated with this object.
    #[br(args(&asset_type, string_heap, heap_pointer))]
    #[bw(args(string_heap, heap_pointer))]
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
    pub(crate) fn read<T: Read + Seek>(
        endianness: Endian,
        cursor: &mut T,
        data_heap: &StringHeap,
        string_heap: &StringHeap,
    ) -> BinResult<Layer> {
        let old_pos = cursor.stream_position()?;

        let header =
            LayerHeader::read_options(cursor, endianness, (endianness, data_heap, string_heap))?;

        let mut objects = Vec::new();
        // read instance objects
        {
            let mut instance_offsets = vec![0i32; header.instance_object_count as usize];
            for i in 0..header.instance_object_count {
                instance_offsets[i as usize] = cursor.read_type_args::<i32>(endianness, ())?;
            }

            for i in 0..header.instance_object_count {
                cursor.seek(SeekFrom::Start(
                    old_pos
                        + header.instance_object_offset as u64
                        + instance_offsets[i as usize] as u64,
                ))?;

                objects.push(InstanceObject::read_options(
                    cursor,
                    endianness,
                    (string_heap,),
                )?);
            }
        }

        Ok(Layer { header, objects })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_layerheader_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<LayerHeader, { LayerHeader::SIZE }>();
    }
}
