// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused)]

use std::io::Cursor;

use crate::common::Platform;
use crate::common_file_operations::{read_string, read_string_until_null};
use crate::{ByteSpan, ReadableFile};
use binrw::{BinRead, BinResult, VecArgs};
use binrw::{BinReaderExt, binread, binrw};

#[binread]
#[derive(Debug)]
struct AvfxBlock<T: BinRead + std::fmt::Debug>
where
    T: for<'a> BinRead<Args<'a> = ()>,
{
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = write_string)]
    #[br(map = read_string)]
    name: String,
    #[br(map = |x: u32| x.div_ceil(4) * 4)] // Align to 4 byte boundary
    size: u32,
    data: T,
}

#[binrw::parser(reader, endian)]
fn read_block<T>(magic: &str) -> BinResult<T>
where
    T: for<'a> BinRead<Args<'a> = ()> + std::fmt::Debug,
{
    let block: AvfxBlock<T> = reader.read_type(endian)?;
    if block.name != magic.chars().rev().collect::<String>() {
        return Err(binrw::Error::BadMagic {
            pos: reader.stream_position()?,
            found: Box::new(block.name),
        });
    }
    // TODO: check size

    Ok(block.data)
}

#[binread]
#[br(import(magic: &str))]
#[derive(Debug)]
struct PlaceholderBlock {
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = write_string)]
    #[br(map = read_string)]
    #[br(assert(name == magic.chars().rev().collect::<String>()), err_context("expected {}", magic))]
    name: String,
    #[br(map = |x: u32| x.div_ceil(4) * 4)] // Align to 4 byte boundary
    size: u32,
    #[brw(pad_size_to = size)]
    #[br(count = size)]
    data: Vec<u8>,
}

// FIXME: these are flags maybe? VFXEditor has them marked as such
#[binrw]
#[brw(repr = u32)]
#[derive(Debug)]
pub enum DrawLayer {
    Screen = 0,
    BaseUpper = 1,
    Base = 2,
    BaseLower = 3,
    InWater = 4,
    BeforeCloud = 5,
    BehindCloud = 6,
    BeforeSky = 7,
    PostUI = 8,
    PrevUI = 9,
    FitWater = 10,
    Max = 11,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, PartialEq)]
pub enum DirectionalLightSource {
    None = 0,
    InLocal = 1,
    InGame = 2,
    Max = 3,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug)]
pub enum DrawOrder {
    Default = 0,
    Reverse = 1,
    Depth = 2,
    Max = 3,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug)]
pub enum PointLightSource {
    Default = 0,
    Reverse = 1,
    Depth = 2,
    Max = 3,
}

#[binread]
#[derive(Debug)]
pub struct EmitVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [u8; 4], // TODO: parse correctly
}

#[binread]
#[derive(Debug)]
pub struct DrawVertex {
    pub position: [u16; 4], // TODO: parse correctly
    pub normal: [u8; 4],
    pub tangent: [u8; 4],
    pub color: [u8; 4],
    pub uv: [u16; 4],
}

#[binread]
#[derive(Debug)]
pub struct DrawTriangle {
    pub indices: [i16; 3],
}

#[binread]
#[derive(Debug)]
pub struct AvfxEmitterModel {
    #[br(parse_with = read_block_sized_array, args("VNum"))]
    pub vertex_numbers: Vec<u16>,
    #[br(parse_with = read_block_sized_array, args("VEmt"))]
    pub vertices: Vec<EmitVertex>,
}

#[binread]
#[derive(Debug)]
pub struct AvfxDrawModel {
    #[br(parse_with = read_block_sized_array, args("VDrw"))]
    pub vertices: Vec<DrawVertex>,
    #[br(parse_with = read_block_sized_array, args("VIdx"))]
    pub triangles: Vec<DrawTriangle>,
}

#[binrw::parser(reader, endian)]
fn read_block_sized_array<T>(magic: &str) -> BinResult<Vec<T>>
where
    T: for<'a> BinRead<Args<'a> = ()> + std::fmt::Debug + 'static,
{
    let block: PlaceholderBlock = reader.read_type_args(endian, (magic,))?;
    if block.name != magic.chars().rev().collect::<String>() {
        return Err(binrw::Error::BadMagic {
            pos: reader.stream_position()?,
            found: Box::new(block.name),
        });
    }
    // TODO: check size

    let count = block.data.len() / std::mem::size_of::<T>();
    let mut array_cursor = Cursor::new(block.data);
    array_cursor.read_type_args(endian, VecArgs::builder().count(count).finalize())
}

#[binread]
#[derive(Debug)]
pub struct AvfxTexture {
    #[br(parse_with = read_string_until_null)]
    path: String,
}

/// Animated VFX file, usually with the `.avfx` file extension.
///
/// This is used for the animated VFX effects in-game.
#[binread]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Avfx {
    #[br(parse_with = read_block, args("Ver"))]
    version: u32,

    #[br(parse_with = read_block, args("bDFP"))]
    is_delay_fast_particle: u32,
    #[br(parse_with = read_block, args("bFG"))]
    is_fit_ground: u32,
    #[br(parse_with = read_block, args("bTS"))]
    is_transform_skip: u32,
    #[br(parse_with = read_block, args("bASH"))]
    is_all_stop_on_hide: u32,
    #[br(parse_with = read_block, args("bCBC"))]
    can_be_clipped_out: u32,

    #[br(parse_with = read_block, args("bCul"))]
    clip_box_enabled: u32,
    #[br(parse_with = read_block, args("CBPx"))]
    clip_box_x: f32,
    #[br(parse_with = read_block, args("CBPy"))]
    clip_box_y: f32,
    #[br(parse_with = read_block, args("CBPz"))]
    clip_box_z: f32,
    #[br(parse_with = read_block, args("CBSx"))]
    clip_box_size_x: f32,
    #[br(parse_with = read_block, args("CBSy"))]
    clip_box_size_y: f32,
    #[br(parse_with = read_block, args("CBSz"))]
    clip_box_size_z: f32,

    #[br(parse_with = read_block, args("ZBMs"))]
    bias_z_max_scale: f32,
    #[br(parse_with = read_block, args("ZBMd"))]
    bias_z_max_distance: f32,

    #[br(parse_with = read_block, args("bCmS"))]
    is_camera_space: u32,
    #[br(parse_with = read_block, args("bFEL"))]
    is_full_env_light: u32,
    #[br(parse_with = read_block, args("bOSt"))]
    clip_own_setting: u32,
    #[br(parse_with = read_block, args("SPFR"))]
    soft_particle_fade_range: f32,
    #[br(parse_with = read_block, args("SKO"))]
    soft_key_offset: f32,

    #[br(parse_with = read_block, args("DwLy"))]
    draw_layer: DrawLayer,
    #[br(parse_with = read_block, args("DwOT"))]
    draw_order: DrawOrder,

    #[br(parse_with = read_block, args("DLST"))]
    directional_light_source: DirectionalLightSource,
    #[br(parse_with = read_block, args("PL1S"))]
    point_light1: DirectionalLightSource,
    #[br(parse_with = read_block, args("PL2S"))]
    point_light2: DirectionalLightSource,

    #[br(parse_with = read_block, args("RvPx"))]
    revised_values_position_x: f32,
    #[br(parse_with = read_block, args("RvPy"))]
    revised_values_position_y: f32,
    #[br(parse_with = read_block, args("RvPz"))]
    revised_values_position_z: f32,
    #[br(parse_with = read_block, args("RvRx"))]
    revised_values_rotation_x: f32,
    #[br(parse_with = read_block, args("RvRy"))]
    revised_values_rotation_y: f32,
    #[br(parse_with = read_block, args("RvRz"))]
    revised_values_rotation_z: f32,
    #[br(parse_with = read_block, args("RvSx"))]
    revised_values_scale_x: f32,
    #[br(parse_with = read_block, args("RvSy"))]
    revised_values_scale_y: f32,
    #[br(parse_with = read_block, args("RvSz"))]
    revised_values_scale_z: f32,
    #[br(parse_with = read_block, args("RvR"))]
    revised_values_red: f32,
    #[br(parse_with = read_block, args("RvG"))]
    revised_values_green: f32,
    #[br(parse_with = read_block, args("RvB"))]
    revised_values_blue: f32,

    #[br(parse_with = read_block, args("AFXe"))]
    fade_enabled_x: u32,
    #[br(parse_with = read_block, args("AFXi"))]
    fade_inner_x: f32,
    #[br(parse_with = read_block, args("AFXo"))]
    fade_outer_x: f32,

    #[br(parse_with = read_block, args("AFYe"))]
    fade_enabled_y: u32,
    #[br(parse_with = read_block, args("AFYi"))]
    fade_inner_y: f32,
    #[br(parse_with = read_block, args("AFYo"))]
    fade_outer_y: f32,

    #[br(parse_with = read_block, args("AFZe"))]
    fade_enabled_z: u32,
    #[br(parse_with = read_block, args("AFZi"))]
    fade_inner_z: f32,
    #[br(parse_with = read_block, args("AFZo"))]
    fade_outer_z: f32,

    #[br(parse_with = read_block, args("bGFE"))]
    global_fog_enabled: u32,
    #[br(parse_with = read_block, args("GFIM"))]
    global_fog_influence: f32,

    #[br(parse_with = read_block, args("bLTS"), if(directional_light_source == DirectionalLightSource::InLocal))]
    lts_enabled: u32,

    #[br(parse_with = read_block, args("ScCn"))]
    scheduler_count: u32,
    #[br(parse_with = read_block, args("TlCn"))]
    timeline_count: u32,
    #[br(parse_with = read_block, args("EmCn"))]
    emitter_count: u32,
    #[br(parse_with = read_block, args("PrCn"))]
    particle_count: u32,
    #[br(parse_with = read_block, args("EfCn"))]
    effector_count: u32,
    #[br(parse_with = read_block, args("BdCn"))]
    binder_count: u32,
    #[br(parse_with = read_block, args("TxCn"))]
    texture_count: u32,
    #[br(parse_with = read_block, args("MdCn"))]
    model_count: u32,

    #[br(args { inner: ("Schd",) }, count = scheduler_count)]
    schedulers: Vec<PlaceholderBlock>,
    #[br(args { inner: ("TmLn",) }, count = timeline_count)]
    timelines: Vec<PlaceholderBlock>,
    #[br(args { inner: ("Emit",) }, count = emitter_count)]
    emitters: Vec<PlaceholderBlock>,
    #[br(args { inner: ("Ptcl",) }, count = particle_count)]
    particles: Vec<PlaceholderBlock>,
    #[br(args { inner: ("Bind",) }, count = binder_count)]
    binders: Vec<PlaceholderBlock>,
    #[br(parse_with = read_block_array, args("Tex", texture_count))]
    pub textures: Vec<AvfxTexture>,
    #[br(parse_with = read_block_pair_array, args("Modl", model_count))]
    pub models: Vec<(AvfxEmitterModel, AvfxDrawModel)>,
    // TODO: selectively enabled, but with what?
    // #[br(parse_with = read_block, args("bAGS"))]
    // ags_enabled: u32,
    // #[br(parse_with = read_block, args("bOSE"))]
    // ose: u32,
    // #[br(parse_with = read_block, args("NCB"))]
    // near_clip_begin: f32,
    // #[br(parse_with = read_block, args("NCE"))]
    // near_clip_end: f32,
    // #[br(parse_with = read_block, args("FCB"))]
    // far_clip_begin: f32,
    // #[br(parse_with = read_block, args("FCE"))]
    // far_clip_end: f32,
}

#[binrw::parser(reader, endian)]
fn read_block_pair_array<T, E>(magic: &str, count: u32) -> BinResult<Vec<(T, E)>>
where
    T: for<'a> BinRead<Args<'a> = ()> + std::fmt::Debug + 'static,
    E: for<'a> BinRead<Args<'a> = ()> + std::fmt::Debug + 'static,
{
    let mut res = Vec::new();
    for _ in 0..count / 2 {
        let block: PlaceholderBlock = reader.read_type_args(endian, (magic,))?;
        if block.name != magic.chars().rev().collect::<String>() {
            return Err(binrw::Error::BadMagic {
                pos: reader.stream_position()?,
                found: Box::new(block.name),
            });
        }

        let mut array_cursor = Cursor::new(block.data);
        let first = array_cursor.read_type(endian)?;

        let block: PlaceholderBlock = reader.read_type_args(endian, (magic,))?;
        if block.name != magic.chars().rev().collect::<String>() {
            return Err(binrw::Error::BadMagic {
                pos: reader.stream_position()?,
                found: Box::new(block.name),
            });
        }

        let mut array_cursor = Cursor::new(block.data);
        let second = array_cursor.read_type(endian)?;

        res.push((first, second));
    }
    Ok(res)
}

#[binrw::parser(reader, endian)]
fn read_block_array<T>(magic: &str, count: u32) -> BinResult<Vec<T>>
where
    T: for<'a> BinRead<Args<'a> = ()> + std::fmt::Debug + 'static,
{
    let mut res = Vec::new();
    for _ in 0..count {
        let block: PlaceholderBlock = reader.read_type_args(endian, (magic,))?;
        if block.name != magic.chars().rev().collect::<String>() {
            return Err(binrw::Error::BadMagic {
                pos: reader.stream_position()?,
                found: Box::new(block.name),
            });
        }

        let mut array_cursor = Cursor::new(block.data);
        res.push(array_cursor.read_type(endian)?)
    }
    Ok(res)
}

impl ReadableFile for Avfx {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let header =
            AvfxBlock::<Avfx>::read_options(&mut cursor, platform.endianness(), ()).ok()?;
        Some(header.data)
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Avfx>();
    }
}
