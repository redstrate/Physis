// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Read, Seek, SeekFrom};

use binrw::{BinRead, binread, BinReaderExt};
use binrw::binrw;
use crate::ByteSpan;

// From https://github.com/NotAdam/Lumina/tree/40dab50183eb7ddc28344378baccc2d63ae71d35/src/Lumina/Data/Parsing/Layer

// TODO: convert these all to magic
#[binrw]
#[repr(i32)]
#[derive(Debug, PartialEq)]
enum LayerEntryType
{
    #[brw(magic = 0x0i32)]
    AssetNone,
    #[brw(magic = 0x1i32)]
    BG {
        asset_path_offset: u32,
        collision_asset_path_offset: i32,
        collision_type: ModelCollisionType,
        attribute_mask: u32,
        attribute: u32,
        collision_config: u32,
        is_visible: u8,
        render_shadow_enabled: u8,
        render_light_shadow_enabled: u8,
        padding: u8,
        render_model_clip_range: f32
    },
    #[brw(magic = 0x2i32)]
    Attribute,
    #[brw(magic = 0x3i32)]
    LayLight,
    #[brw(magic = 0x4i32)]
    VFX,
    #[brw(magic = 0x5i32)]
    PositionMarker,
    #[brw(magic = 0x6i32)]
    SharedGroup,
    Sound = 0x7, //  //
    EventNPC = 0x8, //  //
    BattleNPC = 0x9, //  //
    RoutePath = 0xA,
    Character = 0xB,
    Aetheryte = 0xC, //  //
    EnvSet = 0xD, //  //
    Gathering = 0xE, //  //
    HelperObject = 0xF, //
    Treasure = 0x10, //  //
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
    Weapon = 0x27, //
    PopRange = 0x28, //  //
    ExitRange = 0x29, //  //
    LVB = 0x2A,
    MapRange = 0x2B, //  //
    NaviMeshRange = 0x2C, //  //
    EventObject = 0x2D, //  //
    DemiHuman = 0x2E,
    EnvLocation = 0x2F, //  //
    ControlPoint = 0x30,
    EventRange = 0x31, //?     //
    RestBonusRange = 0x32,
    QuestMarker = 0x33, //      //
    Timeline = 0x34,
    ObjectBehaviorSet = 0x35,
    Movie = 0x36,
    ScenarioExd = 0x37,
    ScenarioText = 0x38,
    CollisionBox = 0x39, //  //
    DoorRange = 0x3A, //
    LineVFX = 0x3B, //  //
    SoundEnvSet = 0x3C,
    CutActionTimeline = 0x3D,
    CharaScene = 0x3E,
    CutAction = 0x3F,
    EquipPreset = 0x40,
    ClientPath = 0x41, //      //
    ServerPath = 0x42, //      //
    GimmickRange = 0x43, //      //
    TargetMarker = 0x44, //      //
    ChairMarker = 0x45, //      //
    ClickableRange = 0x46, //
    PrefetchRange = 0x47, //      //
    FateRange = 0x48, //      //
    PartyMember = 0x49,
    KeepRange = 0x4A, //
    SphereCastRange = 0x4B,
    IndoorObject = 0x4C,
    OutdoorObject = 0x4D,
    EditGroup = 0x4E,
    StableChocobo = 0x4F,
    MaxAssetType = 0x50,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum DoorState
{
    Auto = 0x1,
    Open = 0x2,
    Closed = 0x3,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum RotationState
{
    Rounding = 0x1,
    Stopped = 0x2,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum TransformState
{
    TransformStatePlay = 0x0,
    TransformStateStop = 0x1,
    TransformStateReplay = 0x2,
    TransformStateReset = 0x3,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum ColourState
{
    ColorStatePlay = 0x0,
    ColorStateStop = 0x1,
    ColorStateReplay = 0x2,
    ColorStateReset = 0x3,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum TriggerBoxShape
{
    TriggerBoxShapeBox = 0x1,
    TriggerBoxShapeSphere = 0x2,
    TriggerBoxShapeCylinder = 0x3,
    TriggerBoxShapeBoard = 0x4,
    TriggerBoxShapeMesh = 0x5,
    TriggerBoxShapeBoardBothSides = 0x6,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
enum ModelCollisionType
{
    None = 0x0,
    Replace = 0x1,
    Box = 0x2,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum LightType
{
    None = 0x0,
    Directional = 0x1,
    Point = 0x2,
    Spot = 0x3,
    Plane = 0x4,
    Line = 0x5,
    Specular = 0x6,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum PointLightType
{
    Sphere = 0x0,
    Hemisphere = 0x1,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum PositionMarkerType
{
    DebugZonePop = 0x1,
    DebugJump = 0x2,
    NaviMesh = 0x3,
    LQEvent = 0x4,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum EnvSetShape
{
    EnvShapeEllipsoid = 0x1,
    EnvShapeCuboid = 0x2,
    EnvShapeCylinder = 0x3,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum HelperObjectType
{
    ProxyActor = 0x0,
    NullObject = 0x1,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum TargetType
{
    None = 0x0,
    ENPCInstanceID = 0x1,
    Player = 0x2,
    PartyMember = 0x3,
    ENPCDirect = 0x4,
    BNPCDirect = 0x5,
    BGObjInstanceID = 0x6,
    SharedGroupInstanceID = 0x7,
    BGObj = 0x8,
    SharedGroup = 0x9,
    Weapon = 0xA,
    StableChocobo = 0xB,
    AllianceMember = 0xC,
    Max = 0xD,
}

#[binread]
#[derive(Debug, PartialEq)]
enum PopType
{
    #[br(magic = 0x1u8)]
    PC = 0x1,
    #[br(magic = 0x2u8)]
    NPC,
    #[br(magic = 0x2u8)]
    BNPC,
    #[br(magic = 0x3u8)]
    Content,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum ExitType
{
    ZoneLine = 0x1,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum RangeType
{
    Type01 = 0x1,
    Type02 = 0x2,
    Type03 = 0x3,
    Type04 = 0x4,
    Type05 = 0x5,
    Type06 = 0x6,
    Type07 = 0x7,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum LineStyle
{
    Red = 0x1,
    Blue = 0x2,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum GimmickType
{
    Fishing = 0x1,
    Content = 0x2,
    Room = 0x3,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum TargetMarkerType
{
    UiTarget = 0x0,
    UiNameplate = 0x1,
    LookAt = 0x2,
    BodyDyn = 0x3,
    Root = 0x4,
}

//For ChairMarker
#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum ObjectType
{
    ObjectChair = 0x0,
    ObjectBed = 0x1,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum CharacterSize
{
    DefaultSize = 0x0,
    VerySmall = 0x1,
    Small = 0x2,
    Medium = 0x3,
    Large = 0x4,
    VeryLarge = 0x5,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum DrawHeadParts
{
    Default = 0x0,
    ForceOn = 0x1,
    ForceOff = 0x2,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum RotationType
{
    NoRotate = 0x0,
    AllAxis = 0x1,
    YAxisOnly = 0x2,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum MovePathMode
{
    None = 0x0,
    SharedGroupAction = 0x1,
    Timeline = 0x2,
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
enum LayerSetReferencedType
{
    All = 0x0,
    Include = 0x1,
    Exclude = 0x2,
    Undetermined = 0x3,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum SoundEffectType
{
    Point = 0x3,
    PointDir = 0x4,
    Line = 0x5,
    PolyLine = 0x6,
    Surface = 0x7,
    BoardObstruction = 0x8,
    BoxObstruction = 0x9,
    PolyLineObstruction = 0xB,
    PolygonObstruction = 0xC,
    LineExtController = 0xD,
    Polygon = 0xE,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct LayerHeader {
    layer_id: u32,
    name_offset: u32,

    instance_object_offset: i32,
    instance_object_count: i32,

    tool_mode_visible: u8,
    tool_mode_read_only: u8,

    is_bush_layer: u8,
    ps3_visible: u8,
    layer_set_referenced_list_offset: i32,
    festival_id: u16,
    festival_phase_id: u16,
    is_temporary: u8,
    is_housing: u8,
    version_mask: u16,

    #[br(temp)]
    padding: u32,

    ob_set_referenced_list: i32,
    ob_set_referenced_list_count: i32,
    ob_set_enable_referenced_list: i32,
    ob_set_enable_referenced_list_count: i32,
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct LayerSetReferencedList {
    referenced_type: LayerSetReferencedType,
    layer_sets: i32,
    layer_set_count: i32
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct LgbHeader {
    #[br(count = 4)]
    file_id: Vec<u8>,
    file_size: i32,
    total_chunk_count: i32
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct LayerChunk {
    #[br(count = 4)]
    chunk_id: Vec<u8>,
    chunk_size: i32,
    layer_group_id: i32,
    name_offset: u32,
    layer_offset: i32,
    layer_count: i32
}

#[binread]
#[derive(Debug)]
#[br(little)]
struct InstanceObject {
    asset_type: LayerEntryType,
    instance_id: u32,
    name_offset: u32
}

#[derive(Debug)]
pub struct Layer {

}

impl Layer {
    /// Reads an existing PBD file
    pub fn from_existing(buffer: ByteSpan) -> Option<Layer> {
        let mut cursor = Cursor::new(buffer);

        let mut file_header = LgbHeader::read(&mut cursor).unwrap();

        let mut chunk_header = LayerChunk::read(&mut cursor).unwrap();

        let old_pos = cursor.position();

        let mut layer_offsets = vec![0i32; chunk_header.layer_count as usize];
        for i in 0.. chunk_header.layer_count {
            layer_offsets[i as usize] = cursor.read_le::<i32>().unwrap();
        }

        for i in 0.. chunk_header.layer_count {
            cursor.seek(SeekFrom::Start(old_pos + layer_offsets[i as usize] as u64)).unwrap();

            let old_pos = cursor.position();

            let mut header = LayerHeader::read(&mut cursor).unwrap();

            println!("{:#?}", header);

            cursor.seek(SeekFrom::Start(old_pos + header.instance_object_offset as u64)).unwrap();

            let mut instance_offsets = vec![0i32; header.instance_object_count as usize];
            for i in 0..header.instance_object_count {
                instance_offsets[i as usize] = cursor.read_le::<i32>().unwrap();
            }

            cursor.seek(SeekFrom::Start(old_pos + header.layer_set_referenced_list_offset as u64)).unwrap();
            let mut referenced_list = LayerSetReferencedList::read(&mut cursor).unwrap();

            for i in 0..header.instance_object_count {
                cursor.seek(SeekFrom::Start(old_pos + header.instance_object_offset as u64 + instance_offsets[i as usize] as u64)).unwrap();

                let instance_object = InstanceObject::read(&mut cursor).unwrap();
                println!("{:#?}", instance_object);
            }
        }

        Some(Layer {
            //header
        })
    }
}
