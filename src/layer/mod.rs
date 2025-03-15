// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::ByteSpan;
use crate::common_file_operations::{read_bool_from, string_from_offset};
use binrw::binrw;
use binrw::{BinRead, BinReaderExt, binread};

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

// From https://github.com/NotAdam/Lumina/tree/40dab50183eb7ddc28344378baccc2d63ae71d35/src/Lumina/Data/Parsing/Layer
// Also see https://github.com/aers/FFXIVClientStructs/blob/6b62122cae38bfbc016bf697bef75f80f37abac1/FFXIVClientStructs/FFXIV/Client/LayoutEngine/ILayoutInstance.cs

// TODO: convert these all to magic
#[binrw]
#[brw(repr = i32)]
#[repr(i32)]
#[derive(Debug, PartialEq)]
enum LayerEntryType {
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
}

#[binread]
#[derive(Debug)]
#[br(import(magic: &LayerEntryType))]
enum LayerEntryData {
    #[br(pre_assert(*magic == LayerEntryType::BG))]
    BG(BGInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::LayLight))]
    LayLight(LightInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Vfx))]
    Vfx(VFXInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::PositionMarker))]
    PositionMarker(PositionMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::SharedGroup))]
    SharedGroup(SharedGroupInstance),
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
    EnvLocation(EnvLocationObject),
    #[br(pre_assert(*magic == LayerEntryType::EventRange))]
    EventRange(EventRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::QuestMarker))]
    QuestMarker(QuestMarkerInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::CollisionBox))]
    CollisionBox(CollisionBoxInstanceObject),
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
    #[br(pre_assert(*magic == LayerEntryType::PrefetchRange))]
    PrefetchRange(PrefetchRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::FateRange))]
    FateRange(FateRangeInstanceObject),
    #[br(pre_assert(*magic == LayerEntryType::Unk1))]
    Unk1(),
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct VFXInstanceObject {
    asset_path_offset: u32,
    soft_particle_fade_range: f32,
    padding: u32,
    color: Color,
    #[br(map = read_bool_from::<u8>)]
    auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    no_far_clip: bool,
    padding1: u16,
    fade_near_start: f32,
    fade_near_end: f32,
    fade_far_start: f32,
    fade_far_end: f32,
    z_correct: f32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct GatheringInstanceObject {
    gathering_point_id: u32,
    padding: u32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct TreasureInstanceObject {
    nonpop_init_zone: u8,
    padding1: [u8; 3],
    padding2: [u32; 2],
}

// Unimplemented because I haven't needed it yet:
#[binread]
#[derive(Debug)]
#[br(little)]
struct MapRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct EventInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct EnvLocationObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct EventRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct QuestMarkerInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct CollisionBoxInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct LineVFXInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct ClientPathInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct ServerPathInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct GimmickRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct TargetMarkerInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct ChairMarkerInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct PrefetchRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
struct FateRangeInstanceObject {}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
enum LayerSetReferencedType {
    All = 0x0,
    Include = 0x1,
    Exclude = 0x2,
    Undetermined = 0x3,
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[br(import(start: u64))]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LayerHeader {
    layer_id: u32,

    #[br(parse_with = string_from_offset, args(start))]
    name: String,

    instance_object_offset: i32,
    instance_object_count: i32,

    #[br(map = read_bool_from::<u8>)]
    tool_mode_visible: bool,
    #[br(map = read_bool_from::<u8>)]
    tool_mode_read_only: bool,

    #[br(map = read_bool_from::<u8>)]
    is_bush_layer: bool,
    #[br(map = read_bool_from::<u8>)]
    ps3_visible: bool,
    layer_set_referenced_list_offset: i32,
    festival_id: u16,
    festival_phase_id: u16,
    is_temporary: u8,
    is_housing: u8,
    version_mask: u16,

    #[br(pad_before = 4)]
    ob_set_referenced_list: i32,
    ob_set_referenced_list_count: i32,
    ob_set_enable_referenced_list: i32,
    ob_set_enable_referenced_list_count: i32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LayerSetReferencedList {
    referenced_type: LayerSetReferencedType,
    layer_sets: i32,
    layer_set_count: i32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct OBSetReferenced {
    asset_type: LayerEntryType,
    instance_id: u32,
    ob_set_asset_path_offset: u32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct OBSetEnableReferenced {
    asset_type: LayerEntryType,
    instance_id: u32,
    #[br(map = read_bool_from::<u8>)]
    ob_set_enable: bool,
    #[br(map = read_bool_from::<u8>)]
    ob_set_emissive_enable: bool,
    padding: [u8; 2],
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LgbHeader {
    #[br(count = 4)]
    file_id: Vec<u8>,
    file_size: i32,
    total_chunk_count: i32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LayerChunk {
    #[br(count = 4)]
    chunk_id: Vec<u8>,
    chunk_size: i32,
    layer_group_id: i32,
    name_offset: u32,
    layer_offset: i32,
    layer_count: i32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[br(import(start: u64))]
#[allow(dead_code)] // most of the fields are unused at the moment
struct InstanceObject {
    asset_type: LayerEntryType,
    pub instance_id: u32,
    #[br(parse_with = string_from_offset, args(start))]
    pub name: String,
    pub transform: Transformation,
    #[br(args(&asset_type))]
    pub data: LayerEntryData,
}

#[derive(Debug)]
pub struct Layer {
    pub id: u32,
    pub name: String,
    pub objects: Vec<InstanceObject>,
}

#[derive(Debug)]
pub struct LayerGroup {
    pub layers: Vec<Layer>,
}

impl LayerGroup {
    /// Reads an existing PBD file
    pub fn from_existing(buffer: ByteSpan) -> Option<LayerGroup> {
        let mut cursor = Cursor::new(buffer);

        let file_header = LgbHeader::read(&mut cursor).unwrap();
        if file_header.file_size < 0 || file_header.total_chunk_count < 0 {
            return None;
        }

        let chunk_header = LayerChunk::read(&mut cursor).unwrap();

        let old_pos = cursor.position();

        let mut layer_offsets = vec![0i32; chunk_header.layer_count as usize];
        for i in 0..chunk_header.layer_count {
            layer_offsets[i as usize] = cursor.read_le::<i32>().unwrap();
        }

        let mut layers = Vec::new();

        for i in 0..chunk_header.layer_count {
            cursor
                .seek(SeekFrom::Start(old_pos + layer_offsets[i as usize] as u64))
                .unwrap();

            let old_pos = cursor.position();

            let header = LayerHeader::read_le_args(&mut cursor, (old_pos,)).unwrap();

            let mut objects = Vec::new();
            // read instance objects
            {
                let mut instance_offsets = vec![0i32; header.instance_object_count as usize];
                for i in 0..header.instance_object_count {
                    instance_offsets[i as usize] = cursor.read_le::<i32>().unwrap();
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

                    objects.push(InstanceObject::read_le_args(&mut cursor, (start,)).unwrap());
                }
            }

            // read layer set referenced list
            {
                // NOTE: this casting is INTENTIONALLY like this, layer_set_referenced_list_offset CAN be negative!
                cursor
                    .seek(SeekFrom::Start(
                        (old_pos as i32 + header.layer_set_referenced_list_offset) as u64,
                    ))
                    .unwrap();
                let ref_list = LayerSetReferencedList::read(&mut cursor).unwrap();
            }

            // read ob set referenced
            {
                cursor
                    .seek(SeekFrom::Start(
                        old_pos + header.ob_set_referenced_list as u64,
                    ))
                    .unwrap();
                for _ in 0..header.ob_set_referenced_list_count {
                    OBSetReferenced::read(&mut cursor).unwrap();
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
                    OBSetEnableReferenced::read(&mut cursor).unwrap();
                }
            }

            layers.push(Layer {
                id: header.layer_id,
                name: header.name,
                objects,
            });
        }

        Some(LayerGroup { layers })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        LayerGroup::from_existing(&read(d).unwrap());
    }
}
