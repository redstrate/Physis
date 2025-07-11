// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::common_file_operations::{read_bool_from, write_bool_as, write_string};
use crate::{ByteBuffer, ByteSpan};
use binrw::{BinRead, BinReaderExt, BinWrite, binread};
use binrw::{Endian, Error, binrw};

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

/// "LGB1"
pub const LGB1_ID: u32 = u32::from_le_bytes(*b"LGB1");
/// "LGP1"
pub const LGP1_ID: u32 = u32::from_le_bytes(*b"LGP1");

/// A string that exists in a different location in the file, usually a heap with a bunch of other strings.
#[binrw]
#[br(import(string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Clone, Debug)]
pub struct HeapString {
    #[br(temp)]
    // TODO: this cast is stupid
    #[bw(calc = string_heap.get_free_offset_string(value) as u32)]
    pub offset: u32,
    #[br(calc = string_heap.read_string(r, offset,))]
    #[bw(ignore)]
    pub value: String,
}

#[derive(Debug)]
pub struct StringHeap {
    pub pos: u64,
    pub bytes: Vec<u8>,
    pub free_pos: u64,
}

impl StringHeap {
    pub fn from(pos: u64) -> Self {
        Self {
            pos,
            bytes: Vec::new(),
            free_pos: 0, // unused, so it doesn't matter
        }
    }

    pub fn get_free_offset_args<T>(&mut self, obj: &T) -> i32
    where
        T: for<'a> BinWrite<Args<'a> = (&'a mut StringHeap,)> + std::fmt::Debug,
    {
        // figure out size of it
        let mut buffer = ByteBuffer::new();
        {
            let mut cursor = Cursor::new(&mut buffer);
            obj.write_le_args(&mut cursor, (self,)).unwrap();
        }

        self.bytes.append(&mut buffer);

        let old_pos = self.free_pos;
        self.free_pos += buffer.len() as u64;

        old_pos as i32
    }

    pub fn get_free_offset<T>(&mut self, obj: &T) -> i32
    where
        T: for<'a> BinWrite<Args<'a> = ()> + std::fmt::Debug,
    {
        // figure out size of it
        let mut buffer = ByteBuffer::new();
        {
            let mut cursor = Cursor::new(&mut buffer);
            obj.write_le(&mut cursor).unwrap();
        }

        self.bytes.append(&mut buffer);

        let old_pos = self.free_pos;
        self.free_pos += buffer.len() as u64;

        old_pos as i32
    }

    pub fn get_free_offset_string(&mut self, str: &String) -> i32 {
        let bytes = write_string(str);
        self.get_free_offset(&bytes)
    }

    pub fn read<R, T>(&self, reader: &mut R, offset: i32) -> T
    where
        R: Read + Seek,
        T: for<'a> BinRead<Args<'a> = ()>,
    {
        let old_pos = reader.stream_position().unwrap();
        reader
            .seek(SeekFrom::Start((self.pos as i32 + offset) as u64))
            .unwrap();
        let obj = reader.read_le::<T>().unwrap();
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        obj
    }

    pub fn read_args<R, T>(&self, reader: &mut R, offset: i32) -> T
    where
        R: Read + Seek,
        T: for<'a> BinRead<Args<'a> = (&'a StringHeap,)>,
    {
        let old_pos = reader.stream_position().unwrap();
        reader
            .seek(SeekFrom::Start((self.pos as i32 + offset) as u64))
            .unwrap();
        let obj = reader.read_le_args::<T>((self,)).unwrap();
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        obj
    }

    pub fn read_string<R>(&self, reader: &mut R, offset: u32) -> String
    where
        R: Read + Seek,
    {
        let offset = self.pos + offset as u64;

        let mut string = String::new();

        let old_pos = reader.stream_position().unwrap();

        reader.seek(SeekFrom::Start(offset)).unwrap();
        let mut next_char = reader.read_le::<u8>().unwrap() as char;
        while next_char != '\0' {
            string.push(next_char);
            next_char = reader.read_le::<u8>().unwrap() as char;
        }
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        string
    }
}

impl BinWrite for StringHeap {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> Result<(), Error> {
        self.bytes.write_options(writer, endian, ())?;

        Ok(())
    }
}

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

#[binread]
#[derive(Debug)]
#[br(import(magic: &LayerEntryType, string_heap: &StringHeap))]
pub enum LayerEntryData {
    #[br(pre_assert(*magic == LayerEntryType::BG))]
    BG(#[br(args(string_heap))] BGInstanceObject),
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
    #[br(pre_assert(*magic == LayerEntryType::Unk2))]
    Unk2(),
    #[br(pre_assert(*magic == LayerEntryType::Unk3))]
    Unk3(),
    #[br(pre_assert(*magic == LayerEntryType::Unk4))]
    Unk4(),
    #[br(pre_assert(*magic == LayerEntryType::DoorRange))]
    DoorRange(),
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct VFXInstanceObject {
    pub asset_path_offset: u32,
    #[brw(pad_after = 4)] // padding
    pub soft_particle_fade_range: f32,
    pub color: Color,
    #[br(map = read_bool_from::<u8>)]
    pub auto_play: bool,
    #[br(map = read_bool_from::<u8>)]
    #[brw(pad_after = 2)] // padding
    pub no_far_clip: bool,
    pub fade_near_start: f32,
    pub fade_near_end: f32,
    pub fade_far_start: f32,
    pub fade_far_end: f32,
    pub z_correct: f32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct GatheringInstanceObject {
    #[brw(pad_after = 4)] // padding
    pub gathering_point_id: u32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct TreasureInstanceObject {
    #[brw(pad_after = 11)] // padding
    pub nonpop_init_zone: u8,
}

// Unimplemented because I haven't needed it yet:
#[binread]
#[derive(Debug)]
#[br(little)]
pub struct MapRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct EventInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct EnvLocationObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct EventRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct QuestMarkerInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct CollisionBoxInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct LineVFXInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ClientPathInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ServerPathInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct GimmickRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct TargetMarkerInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct ChairMarkerInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct PrefetchRangeInstanceObject {}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct FateRangeInstanceObject {}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
enum LayerSetReferencedType {
    All = 0x0,
    Include = 0x1,
    Exclude = 0x2,
    Undetermined = 0x3,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
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
#[derive(Debug)]
#[brw(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct LayerSetReferenced {
    pub layer_set_id: u32,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
#[br(import(data_heap: &StringHeap), stream = r)]
#[bw(import(data_heap: &mut StringHeap))]
pub struct LayerSetReferencedList {
    referenced_type: LayerSetReferencedType,
    #[br(temp)]
    #[bw(calc = data_heap.get_free_offset(&layer_sets))]
    layer_set_offset: i32,
    #[bw(calc = layer_sets.len() as i32)]
    pub layer_set_count: i32,

    #[br(count = layer_set_count)]
    #[bw(ignore)]
    pub layer_sets: Vec<LayerSetReferenced>,
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

#[binrw]
#[derive(Debug)]
#[brw(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LgbHeader {
    // Example: "LGB1"
    file_id: u32,
    // File size *including* this header
    file_size: i32,
    total_chunk_count: i32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap))]
#[brw(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LayerChunkHeader {
    chunk_id: u32,
    chunk_size: i32,
    layer_group_id: i32,
    #[brw(args(string_heap))]
    pub name: HeapString,
    layer_offset: i32,
    layer_count: i32,
}

const LAYER_CHUNK_HEADER_SIZE: usize = 24;

#[binread]
#[derive(Debug)]
#[br(little)]
#[br(import(string_heap: &StringHeap))]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct InstanceObject {
    asset_type: LayerEntryType,
    pub instance_id: u32,
    #[br(args(string_heap))]
    pub name: HeapString,
    pub transform: Transformation,
    #[br(args(&asset_type, string_heap))]
    pub data: LayerEntryData,
}

#[derive(Debug)]
pub struct Layer {
    pub header: LayerHeader,
    pub objects: Vec<InstanceObject>,
}

#[derive(Debug)]
pub struct LayerChunk {
    // Example: "LGP1"
    pub chunk_id: u32,
    pub layer_group_id: i32,
    pub name: String,
    pub layers: Vec<Layer>,
}

#[derive(Debug)]
pub struct LayerGroup {
    pub file_id: u32,
    pub chunks: Vec<LayerChunk>,
}

impl LayerGroup {
    /// Reads an existing PBD file
    pub fn from_existing(buffer: ByteSpan) -> Option<LayerGroup> {
        let mut cursor = Cursor::new(buffer);

        let file_header = LgbHeader::read(&mut cursor).unwrap();
        if file_header.file_size <= 0 || file_header.total_chunk_count <= 0 {
            return None;
        }

        // yes, for some reason it begins at 8 bytes in?!?!
        let chunk_string_heap = StringHeap::from(cursor.position() + 8);

        let chunk_header =
            LayerChunkHeader::read_le_args(&mut cursor, (&chunk_string_heap,)).unwrap();
        if chunk_header.chunk_size <= 0 {
            return Some(LayerGroup {
                file_id: file_header.file_id,
                chunks: Vec::new(),
            });
        }

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

            let string_heap = StringHeap::from(old_pos);
            let data_heap = StringHeap::from(old_pos);

            let header =
                LayerHeader::read_le_args(&mut cursor, (&data_heap, &string_heap)).unwrap();

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
                    let string_heap = StringHeap::from(start);

                    objects.push(InstanceObject::read_le_args(&mut cursor, (&string_heap,)).ok()?);
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

            layers.push(Layer { header, objects });
        }

        let layer_chunk = LayerChunk {
            chunk_id: chunk_header.chunk_id,
            layer_group_id: chunk_header.layer_group_id,
            name: chunk_header.name.value,
            layers,
        };

        Some(LayerGroup {
            file_id: file_header.file_id,
            chunks: vec![layer_chunk],
        })
    }

    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);

            // skip header, will be writing it later
            cursor
                .seek(SeekFrom::Start(std::mem::size_of::<LgbHeader>() as u64))
                .unwrap();

            // base offset for deferred data
            let mut data_base = cursor.stream_position().unwrap();

            let mut chunk_data_heap = StringHeap {
                pos: data_base + 4,
                bytes: Vec::new(),
                free_pos: data_base + 4,
            };

            let mut chunk_string_heap = StringHeap {
                pos: data_base + 4,
                bytes: Vec::new(),
                free_pos: data_base + 4,
            };

            // we will write this later, when we have a working string heap
            let layer_chunk_header_pos = cursor.stream_position().unwrap();
            cursor
                .seek(SeekFrom::Current(LAYER_CHUNK_HEADER_SIZE as i64))
                .unwrap();

            // skip offsets for now, they will be written later
            let offset_pos = cursor.position();
            cursor
                .seek(SeekFrom::Current(
                    (std::mem::size_of::<i32>() * self.chunks[0].layers.len()) as i64,
                ))
                .ok()?;

            let mut offsets: Vec<i32> = Vec::new();

            let layer_data_offset = cursor.position();

            // first pass: write layers, we want to get a correct *chunk_data_heap*
            for layer in &self.chunks[0].layers {
                // set offset
                // this is also used to reference positions inside this layer
                let layer_offset = cursor.position() as i32;
                offsets.push(layer_offset);

                layer
                    .header
                    .write_le_args(&mut cursor, (&mut chunk_data_heap, &mut chunk_string_heap))
                    .ok()?;
            }

            // make sure the heaps are at the end of the layer data
            data_base += cursor.stream_position().unwrap() - layer_data_offset;

            // second pass: write layers again, we want to get a correct *chunk_string_heap* now that we know of the size of chunk_data_heap
            chunk_string_heap = StringHeap {
                pos: data_base + 4 + chunk_data_heap.bytes.len() as u64,
                bytes: Vec::new(),
                free_pos: data_base + 4 + chunk_data_heap.bytes.len() as u64,
            };
            chunk_data_heap = StringHeap {
                pos: data_base + 4,
                bytes: Vec::new(),
                free_pos: data_base + 4,
            };

            // write header now, because it has a string
            cursor
                .seek(SeekFrom::Start(layer_chunk_header_pos))
                .unwrap();
            // TODO: support multiple layer chunks
            let layer_chunk = LayerChunkHeader {
                chunk_id: self.chunks[0].chunk_id,
                chunk_size: 24, // double lol
                layer_group_id: self.chunks[0].layer_group_id,
                name: HeapString {
                    value: self.chunks[0].name.clone(),
                },
                layer_offset: 16, // lol
                layer_count: self.chunks[0].layers.len() as i32,
            };
            layer_chunk
                .write_le_args(&mut cursor, (&mut chunk_string_heap,))
                .ok()?;

            // now write the layer data for the final time
            cursor.seek(SeekFrom::Start(layer_data_offset)).unwrap();
            for layer in &self.chunks[0].layers {
                layer
                    .header
                    .write_le_args(&mut cursor, (&mut chunk_data_heap, &mut chunk_string_heap))
                    .ok()?;
            }

            // write the heaps
            chunk_data_heap.write_le(&mut cursor).ok()?;
            chunk_string_heap.write_le(&mut cursor).ok()?;

            // write offsets
            assert_eq!(offsets.len(), self.chunks[0].layers.len());
            cursor.seek(SeekFrom::Start(offset_pos)).ok()?;
            for offset in offsets {
                offset.write_le(&mut cursor).ok()?;
            }
        }

        let file_size = buffer.len() as i32;

        {
            let mut cursor = Cursor::new(&mut buffer);

            // write the header, now that we now the file size
            cursor.seek(SeekFrom::Start(0)).ok()?;
            let lgb_header = LgbHeader {
                file_id: self.file_id,
                file_size,
                total_chunk_count: self.chunks.len() as i32,
            };
            lgb_header.write_le(&mut cursor).ok()?;
        }

        Some(buffer)
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

    #[test]
    fn read_empty_planlive() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty_planlive.lgb");

        let lgb = LayerGroup::from_existing(&read(d).unwrap()).unwrap();
        assert_eq!(lgb.file_id, LGB1_ID);
        assert_eq!(lgb.chunks.len(), 1);

        let chunk = &lgb.chunks[0];
        assert_eq!(chunk.chunk_id, LGP1_ID);
        assert_eq!(chunk.layer_group_id, 261);
        assert_eq!(chunk.name, "PlanLive".to_string());
        assert!(chunk.layers.is_empty());
    }

    #[test]
    fn write_empty_planlive() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty_planlive.lgb");

        let good_lgb_bytes = read(d).unwrap();

        let lgb = LayerGroup {
            file_id: LGB1_ID,
            chunks: vec![LayerChunk {
                chunk_id: LGP1_ID,
                layer_group_id: 261,
                name: "PlanLive".to_string(),
                layers: Vec::new(),
            }],
        };
        assert_eq!(lgb.write_to_buffer().unwrap(), good_lgb_bytes);
    }
}
