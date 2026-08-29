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
pub use bg::{
    AnalyticCollider, AnalyticColliderType, BgPartInstanceObject, ModelCollisionType, ShadowMode,
    WeaponInstanceObject,
};

mod collision;
pub use collision::{
    CollisionAttributes, CollisionBoxInstanceObject, CullingBoxInstanceObject,
    TriggerBoxInstanceObject, TriggerBoxShape,
};

mod env;
pub use env::{EnvLocationObject, EnvSetShape, EnvSpaceInstanceObject};

mod event;
pub use event::EventObjectInstanceObject;

mod gathering;
pub use gathering::GatheringInstanceObject;

mod light;
pub use light::{LightInstanceObject, LightShape, PointLightType};

mod marker;
pub use marker::{
    ChairMarkerInstanceObject, ChairType, PositionMarkerInstanceObject, PositionMarkerType,
    QuestMarkerInstanceObject, TargetMarkerInstanceObject, TargetMarkerType,
};

mod npc;
pub use npc::{BattleNpcInstanceObject, CharacterInstanceObject, EventNpcInstanceObject};

mod path;
pub use path::{
    ClientPathInstanceObject, PathControlPoint, PathInstanceObject, ServerPathInstanceObject,
};

mod range;
pub use range::{
    ClickableRangeInstanceObject, DoorRangeInstanceObject, EventEffectRangeInstanceObject,
    EventRangeInstanceObject, ExitRangeInstanceObject, ExitType, FateRangeInstanceObject,
    GameContentsRangeInstanceObject, GimmickRangeInstanceObject, MapRangeInstanceObject,
    PopRangeInstanceObject, PopType, PrefetchRangeInstanceObject, RangeShape,
    ShowHideRangeInstanceObject, WaterRangeInstanceObject,
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
pub use vfx::{
    DecalInstanceObject, LineStyle, LineVFXInstanceObject, VfxInstanceObject,
    VolumetricCloudInstanceObject,
};

/// Base struct for objects that refer to game data.
#[binrw]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct GameObjectInstanceObject {
    /// What sheet this ID refers to depends on the kind of [InstanceObject] this is.
    pub base_id: u32,
}

// From https://github.com/NotAdam/Lumina/tree/40dab50183eb7ddc28344378baccc2d63ae71d35/src/Lumina/Data/Parsing/Layer
// Also see https://github.com/aers/FFXIVClientStructs/blob/6b62122cae38bfbc016bf697bef75f80f37abac1/FFXIVClientStructs/FFXIV/Client/LayoutEngine/ILayoutInstance.cs
// Note that this doesn't include *everything*, only the things actually read by the client from an LGB file. FFXIVClientStructs also covers stuff that only exists in-memory during gameplay.
#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq, Copy, Clone, Default)]
pub enum LayerEntryType {
    /// This represents nothing. It's also used as a temporary fallback for any unknown objects.
    #[default]
    Unknown = 0,
    /// Background model object that can have collision.
    BgPart = 1,
    /// Light object.
    Light = 3,
    /// Visual effect object.
    Vfx = 4,
    /// Debug information.
    ///
    /// This is stripped out of retail data, and is not used by the client.
    PositionMarker = 5,
    /// Instance of a shared group object.
    SharedGroup = 6,
    /// Sound object.
    Sound = 7,
    /// Event NPC object.
    EventNPC = 8,
    /// Battle NPC object.
    ///
    /// This is stripped out of retail data, and is not used by the client.
    BattleNPC = 9,
    /// Aetheryte object.
    Aetheryte = 12,
    /// Unknown object.
    EnvSpace = 13,
    /// Gathering point.
    ///
    /// This is stripped out of retail data, and is not used by the client.
    Gathering = 14,
    /// Treasure object.
    Treasure = 16,
    /// Displays a weapon model.
    Weapon = 39,
    /// Generic range for characters to spawn in.
    PopRange = 40,
    /// Zone transitions (the visible part is probably LineVFX?)
    ExitRange = 41,
    /// Used to demarcate various aspects of the map, such as location or the BGM used.
    MapRange = 43,
    /// Unknown object.
    NaviMeshRange = 44,
    /// Event object.
    EventObject = 45,
    /// Unknown object.
    EnvLocation = 47,
    /// Generic areas for events to use like FATEs or cutscene triggers.
    EventRange = 49,
    /// Unknown object.
    QuestMarker = 51,
    /// Unknown object.
    CollisionBox = 57,
    /// A hint used to animate the opening of doors.
    DoorRange = 58,
    /// Generic VFX that are those dotted lines used for zone transitions and boundaries.
    LineVFX = 59,
    /// Path object that objects and characters can follow.
    ClientPath = 65,
    /// Path object that objects and characters can follow.
    ///
    /// This is stripped out of retail data, and is not used by the client.
    ServerPath = 66,
    /// Unknown object.
    GimmickRange = 67,
    /// Unknown object.
    TargetMarker = 68,
    /// Marker used to determine where a character to sit or lay down.
    ChairMarker = 69,
    /// Unknown object.
    ClickableRange = 70,
    /// A hint for the client to preload an area.
    PrefetchRange = 71,
    /// Unknown object.
    FateRange = 72,
    /// Unknown object.
    SphereCastRange = 75,
    /// 2D decal drawn on top of a 3D object.
    Decal = 83,
    /// Provides underwater effects like bubbles and swimming.
    WaterRange = 86,
    /// Unknown object.
    ShowHideRange = 87,
    /// Unknown object.
    GameContentsRange = 88,
    /// Unknown object.
    EventEffectRange = 89,
    /// Anything occluded by this object is not drawn.
    CullingBox = 90,
    /// Unknown object.
    Unk91 = 91,
    /// Unknown object.
    Unk92 = 92,
    /// (Presumably) a volumetric cloud.
    ///
    /// This does not currently function in the Dawntrail client.
    VolumetricCloud = 93,
}

impl From<&LayerEntryData> for LayerEntryType {
    fn from(value: &LayerEntryData) -> Self {
        match value {
            LayerEntryData::Unknown => LayerEntryType::Unknown,
            LayerEntryData::BgPart(_) => LayerEntryType::BgPart,
            LayerEntryData::Light(_) => LayerEntryType::Light,
            LayerEntryData::Vfx(_) => LayerEntryType::Vfx,
            LayerEntryData::PositionMarker(_) => LayerEntryType::PositionMarker,
            LayerEntryData::SharedGroup(_) => LayerEntryType::SharedGroup,
            LayerEntryData::Sound(_) => LayerEntryType::Sound,
            LayerEntryData::EventNPC(_) => LayerEntryType::EventNPC,
            LayerEntryData::BattleNPC(_) => LayerEntryType::BattleNPC,
            LayerEntryData::Aetheryte(_) => LayerEntryType::Aetheryte,
            LayerEntryData::EnvSpace(_) => LayerEntryType::EnvSpace,
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
            LayerEntryData::DoorRange(_) => LayerEntryType::DoorRange,
            LayerEntryData::Weapon(_) => LayerEntryType::Weapon,
            LayerEntryData::NaviMeshRange() => LayerEntryType::NaviMeshRange,
            LayerEntryData::SphereCastRange() => LayerEntryType::SphereCastRange,
            LayerEntryData::Decal(_) => LayerEntryType::Decal,
            LayerEntryData::WaterRange(_) => LayerEntryType::WaterRange,
            LayerEntryData::ShowHideRange(_) => LayerEntryType::ShowHideRange,
            LayerEntryData::GameContentsRange(_) => LayerEntryType::GameContentsRange,
            LayerEntryData::EventEffectRange(_) => LayerEntryType::EventEffectRange,
            LayerEntryData::CullingBox(_) => LayerEntryType::CullingBox,
            LayerEntryData::Unk91() => LayerEntryType::Unk91,
            LayerEntryData::Unk92() => LayerEntryType::Unk92,
            LayerEntryData::VolumetricCloud(_) => LayerEntryType::VolumetricCloud,
        }
    }
}

/// Type used to store data for a [InstanceObject].
///
/// The documentation for variants can be found in their individual types or [LayerEntryType].
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[br(import(magic: &LayerEntryType, string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub enum LayerEntryData {
    #[default]
    #[br(pre_assert(*magic == LayerEntryType::Unknown))]
    Unknown,
    #[br(pre_assert(*magic == LayerEntryType::BgPart))]
    BgPart(#[brw(args(string_heap, heap_pointer))] BgPartInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Light))]
    Light(#[brw(args(string_heap, heap_pointer))] LightInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Vfx))]
    Vfx(#[brw(args(string_heap, heap_pointer))] VfxInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::PositionMarker))]
    PositionMarker(PositionMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::SharedGroup))]
    SharedGroup(#[brw(args(string_heap, heap_pointer))] SharedGroupInstance),
    #[br(pre_assert(*magic == LayerEntryType::Sound))]
    Sound(#[brw(args(string_heap, heap_pointer))] SoundInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EventNPC))]
    EventNPC(EventNpcInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::BattleNPC))]
    BattleNPC(BattleNpcInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Aetheryte))]
    Aetheryte(AetheryteInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EnvSpace))]
    EnvSpace(#[brw(args(string_heap, heap_pointer))] EnvSpaceInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Gathering))]
    Gathering(GatheringInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Treasure))]
    Treasure(TreasureInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Weapon))]
    Weapon(#[brw(args(string_heap, heap_pointer))] WeaponInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::PopRange))]
    PopRange(PopRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::ExitRange))]
    ExitRange(ExitRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::MapRange))]
    MapRange(MapRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::NaviMeshRange))]
    NaviMeshRange(),
    #[br(pre_assert(*magic == LayerEntryType::EventObject))]
    EventObject(EventObjectInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EnvLocation))]
    EnvLocation(#[brw(args(string_heap, heap_pointer))] EnvLocationObject),
    #[br(pre_assert(*magic == LayerEntryType::EventRange))]
    EventRange(EventRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::QuestMarker))]
    QuestMarker(QuestMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::CollisionBox))]
    CollisionBox(#[brw(args(string_heap, heap_pointer))] CollisionBoxInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::DoorRange))]
    DoorRange(DoorRangeInstanceObject),
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
    #[br(pre_assert(*magic == LayerEntryType::SphereCastRange))]
    SphereCastRange(),
    #[br(pre_assert(*magic == LayerEntryType::Decal))]
    Decal(#[brw(args(string_heap, heap_pointer))] DecalInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::WaterRange))]
    WaterRange(WaterRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::ShowHideRange))]
    ShowHideRange(ShowHideRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::GameContentsRange))]
    GameContentsRange(GameContentsRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::EventEffectRange))]
    EventEffectRange(EventEffectRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::CullingBox))]
    CullingBox(CullingBoxInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Unk91))]
    Unk91(),
    #[br(pre_assert(*magic == LayerEntryType::Unk92))]
    Unk92(),
    #[br(pre_assert(*magic == LayerEntryType::VolumetricCloud))]
    VolumetricCloud(#[brw(args(string_heap, heap_pointer))] VolumetricCloudInstanceObject),
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

    /// This field should be left at it's default.
    pub instance_object_offset: i32,
    /// This field should be left at it's default.
    pub instance_object_count: i32,

    /// Whether this layer is visible by default. If false, it does not show up in-game.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub active: bool,

    /// Sets 2 in LayerManager::Flags client-side.
    ///
    /// Doesn't seem to be set in any retail layers.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk1: bool,

    /// Always seems to be false in retail layers.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk2: bool,

    /// Unsure of its purpose, but can be true/false for retail layers.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk3: bool,

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

    /// False in all retail layers.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk4: bool,

    /// Sets 4 in LayerManager::Flags client-side.
    ///
    /// Seen true/false in retail layers. Might indicate whether a layer is "indoors"?
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk5: bool,

    /// Has various values in retail layers.
    pub unk6: u16,

    /// Stored in LayerManager 0x1F client-side after casting to a char.
    ///
    /// Seen as 1 or 2 in retail layers, mostly housing but also `bg/ex4/05_zon_z5/shared/for_bg/sgbg_z5r3_a1_bos03.sgb`.
    /// Related to `unk5` in some way, I think.
    #[brw(pad_after = 2)]
    pub unk7: u16,

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
            active: true,
            unk1: Default::default(),
            unk2: Default::default(),
            unk3: Default::default(),
            layer_set_referenced_list: Default::default(),
            festival_id: Default::default(),
            festival_phase_id: Default::default(),
            unk4: Default::default(),
            unk5: Default::default(),
            unk6: Default::default(),
            unk7: Default::default(),
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

/// Represents a single object in [Layer].
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
    #[br(try, temp)]
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
