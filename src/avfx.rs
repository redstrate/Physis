// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use crate::ByteSpan;
use binrw::BinRead;
use binrw::{BinReaderExt, binread, binrw};

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct AvfxHeader {
    name: u32,
    size: u32,
}

#[binread]
#[derive(Debug)]
enum AvfxData {
    #[brw(magic = b"XFVA")]
    AvfxBase,
    #[brw(magic = b"reV\0")]
    Version,
    #[brw(magic = b"PFDb")]
    IsDelayFastParticle,
    #[brw(magic = b"GFb\0")]
    IsFitGround,
    #[brw(magic = b"STb\0")]
    IsTransformSkip,
    #[brw(magic = b"HSAb")]
    IsAllStopOnHide,
    #[brw(magic = b"CBCb")]
    CanBeClippedOut,
    #[brw(magic = b"luCb")]
    ClipBoxEnabled,
    #[brw(magic = b"xPBC")]
    ClipBoxX,
    #[brw(magic = b"yPBC")]
    ClipBoxY,
    #[brw(magic = b"zPBC")]
    ClipBoxZ,
    #[brw(magic = b"xSBC")]
    ClipBoxSizeX,
    #[brw(magic = b"ySBC")]
    ClipBoxSizeY,
    #[brw(magic = b"zSBC")]
    ClipBoxSizeZ,
    #[brw(magic = b"sMBZ")]
    BiasZmaxScale,
    #[brw(magic = b"dMBZ")]
    BiasZmaxDistance,
    #[brw(magic = b"SmCb")]
    IsCameraSpace,
    #[brw(magic = b"LEFb")]
    IsFullEnvLight,
    #[brw(magic = b"tSOb")]
    IsClipOwnSetting,
    #[brw(magic = b"BCN\0")]
    NearClipBegin,
    #[brw(magic = b"ECN\0")]
    NearClipEnd,
    #[brw(magic = b"BCF\0")]
    FarClipBegin,
    #[brw(magic = b"ECF\0")]
    FarClipEnd,
    #[brw(magic = b"RFPS")]
    SoftParticleFadeRange,
    #[brw(magic = b"OKS\0")]
    SoftKeyOffset,
    #[brw(magic = b"yLwD")]
    DrawLayerType,
    #[brw(magic = b"TOwD")]
    DrawOrderType,
    #[brw(magic = b"TSLD")]
    DirectionalLightSourceType,
    #[brw(magic = b"S1LP")]
    PointLightsType1,
    #[brw(magic = b"S2LP")]
    PointLightsType2,
    #[brw(magic = b"xPvR")]
    RevisedValuesPosX,
    #[brw(magic = b"yPvR")]
    RevisedValuesPosY,
    #[brw(magic = b"zPvR")]
    RevisedValuesPosZ,
    #[brw(magic = b"xRvR")]
    RevisedValuesRotX,
    #[brw(magic = b"yRvR")]
    RevisedValuesRotY,
    #[brw(magic = b"zRvR")]
    RevisedValuesRotZ,
    #[brw(magic = b"xSvR")]
    RevisedValuesScaleX,
    #[brw(magic = b"ySvR")]
    RevisedValuesScaleY,
    #[brw(magic = b"zSvR")]
    RevisedValuesScaleZ,
    #[brw(magic = b"RvR\0")]
    RevisedValuesColorR,
    #[brw(magic = b"GvR\0")]
    RevisedValuesColorG,
    #[brw(magic = b"BvR\0")]
    RevisedValuesColorB,
    #[brw(magic = b"eXFA")]
    FadeEnabledX,
    #[brw(magic = b"iXFA")]
    FadeInnerX,
    #[brw(magic = b"oXFA")]
    FadeOuterX,
    #[brw(magic = b"eYFA")]
    FadeEnabledY,
    #[brw(magic = b"iYFA")]
    FadeInnerY,
    #[brw(magic = b"oYFA")]
    FadeOuterY,
    #[brw(magic = b"eZFA")]
    FadeEnabledZ,
    #[brw(magic = b"iZFA")]
    FadeInnerZ,
    #[brw(magic = b"oZFA")]
    FadeOuterZ,
    #[brw(magic = b"EFGb")]
    GlobalFogEnabled,
    #[brw(magic = b"MIFG")]
    GlobalFogInfluence,
    #[brw(magic = b"SGAb")]
    AgsEnabled,
    #[brw(magic = b"STLb")]
    LtsEnabled,
    #[brw(magic = b"nCcS")]
    NumSchedulers,
    #[brw(magic = b"nClT")]
    NumTimelines,
    #[brw(magic = b"nCmE")]
    NumEmitters,
    #[brw(magic = b"nCrP")]
    NumParticles,
    #[brw(magic = b"nCfE")]
    NumEffectors,
    #[brw(magic = b"nCdB")]
    NumBinders,
    #[brw(magic = b"nCxT")]
    NumTextures,
    #[brw(magic = b"nCdM")]
    NumModels,
    #[brw(magic = b"dhcS")]
    Scheduler,
    #[brw(magic = b"nLmT")]
    Timeline,
    #[brw(magic = b"timE")]
    Emitter,
    #[brw(magic = b"lctP")]
    Particle,
    #[brw(magic = b"tcfE")]
    Effector,
    #[brw(magic = b"dniB")]
    Binder,
    #[brw(magic = b"xeT\0")]
    Texture,
    #[brw(magic = b"ldoM")]
    Model,
    Unknown(#[br(temp)] [u8; 4]),
}

#[binread]
#[derive(Debug)]
#[brw(little)]
struct AvfxBlock {
    #[br(pad_before = 4)]
    size: u32,

    #[br(seek_before = SeekFrom::Current(-8))]
    #[br(pad_after = 4)] // skip over size
    data: AvfxData,
}

/// Animated VFX file, usually with the `.avfx` file extension.
///
/// This is used for the animated VFX effects in-game.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Avfx {
    clip_box: [f32; 3],
    clip_box_size: [f32; 3],
    revised_values_position: [f32; 3],
    revised_values_rotation: [f32; 3],
    revised_values_scale: [f32; 3],
    revised_values_color: [f32; 3],

    version: u32,
    draw_layer_type: u32,
    draw_order_type: u32,
    directional_light_source_type: u32,
    point_lights_type1: u32,
    point_lights_type2: u32,

    bias_z_max_scale: f32,
    bias_z_max_distance: f32,
    near_clip_begin: f32,
    near_clip_end: f32,
    fade_inner: [f32; 3],
    fade_outer: [f32; 3],
    far_clip_begin: f32,
    far_clip_end: f32,
    soft_particle_fade_range: f32,
    soft_key_offset: f32,
    global_fog_influence: f32,

    is_delay_fast_particle: bool,
    is_fit_ground: bool,
    is_transform_skip: bool,
    is_all_stop_on_hide: bool,
    can_be_clipped_out: bool,
    clip_box_enabled: bool,
    is_camera_space: bool,
    is_full_env_light: bool,
    is_clip_own_setting: bool,
    fade_enabled_x: bool,
    fade_enabled_y: bool,
    fade_enabled_z: bool,
    global_fog_enabled: bool,
    lts_enabled: bool,
    ags_enabled: bool,

    schedulers: Vec<AvfxBlock>,
    timelines: Vec<AvfxBlock>,
    emitters: Vec<AvfxBlock>,
    particles: Vec<AvfxBlock>,
    effectors: Vec<AvfxBlock>,
    binders: Vec<AvfxBlock>,
    textures: Vec<String>,
    model: Vec<AvfxBlock>,
}

impl Default for Avfx {
    fn default() -> Self {
        Self {
            clip_box: [0.0; 3],
            clip_box_size: [0.0; 3],
            revised_values_position: [0.0; 3],
            revised_values_rotation: [0.0; 3],
            revised_values_scale: [0.0; 3],
            revised_values_color: [0.0; 3],
            version: 0,
            draw_layer_type: 0,
            draw_order_type: 0,
            directional_light_source_type: 0,
            point_lights_type1: 0,
            point_lights_type2: 0,
            bias_z_max_scale: 0.0,
            bias_z_max_distance: 0.0,
            near_clip_begin: 0.0,
            near_clip_end: 0.0,
            fade_inner: [0.0; 3],
            fade_outer: [0.0; 3],
            far_clip_begin: 0.0,
            far_clip_end: 0.0,
            soft_particle_fade_range: 0.0,
            soft_key_offset: 0.0,
            global_fog_influence: 0.0,
            is_delay_fast_particle: false,
            is_fit_ground: false,
            is_transform_skip: false,
            is_all_stop_on_hide: false,
            can_be_clipped_out: false,
            clip_box_enabled: false,
            is_camera_space: false,
            is_full_env_light: false,
            is_clip_own_setting: false,
            fade_enabled_x: false,
            fade_enabled_y: false,
            fade_enabled_z: false,
            global_fog_enabled: false,
            lts_enabled: false,
            ags_enabled: false,
            schedulers: vec![],
            timelines: vec![],
            emitters: vec![],
            particles: vec![],
            effectors: vec![],
            binders: vec![],
            textures: vec![],
            model: vec![],
        }
    }
}

impl Avfx {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let header = AvfxHeader::read(&mut cursor).ok()?;

        let mut avfx = Avfx::default();

        let read_bool = |cursor: &mut Cursor<ByteSpan>| cursor.read_le::<u8>().unwrap() == 1u8;

        let read_uint = |cursor: &mut Cursor<ByteSpan>| cursor.read_le::<u32>().unwrap();

        let read_float = |cursor: &mut Cursor<ByteSpan>| cursor.read_le::<f32>().unwrap();

        while cursor.position() < header.size as u64 {
            let last_pos = cursor.position();
            let block = AvfxBlock::read(&mut cursor).unwrap();
            match block.data {
                AvfxData::AvfxBase => {}
                AvfxData::Version => {
                    avfx.version = read_uint(&mut cursor);
                }
                AvfxData::IsDelayFastParticle => {
                    avfx.is_delay_fast_particle = read_bool(&mut cursor);
                }
                AvfxData::IsFitGround => {
                    avfx.is_fit_ground = read_bool(&mut cursor);
                }
                AvfxData::IsTransformSkip => {
                    avfx.is_transform_skip = read_bool(&mut cursor);
                }
                AvfxData::IsAllStopOnHide => {
                    avfx.is_all_stop_on_hide = read_bool(&mut cursor);
                }
                AvfxData::CanBeClippedOut => {
                    avfx.can_be_clipped_out = read_bool(&mut cursor);
                }
                AvfxData::ClipBoxEnabled => {
                    avfx.clip_box_enabled = read_bool(&mut cursor);
                }
                AvfxData::ClipBoxX => {
                    avfx.clip_box[0] = read_float(&mut cursor);
                }
                AvfxData::ClipBoxY => {
                    avfx.clip_box[1] = read_float(&mut cursor);
                }
                AvfxData::ClipBoxZ => {
                    avfx.clip_box[2] = read_float(&mut cursor);
                }
                AvfxData::ClipBoxSizeX => {
                    avfx.clip_box_size[0] = read_float(&mut cursor);
                }
                AvfxData::ClipBoxSizeY => {
                    avfx.clip_box_size[1] = read_float(&mut cursor);
                }
                AvfxData::ClipBoxSizeZ => {
                    avfx.clip_box_size[2] = read_float(&mut cursor);
                }
                AvfxData::BiasZmaxScale => {
                    avfx.bias_z_max_scale = read_float(&mut cursor);
                }
                AvfxData::BiasZmaxDistance => {
                    avfx.bias_z_max_distance = read_float(&mut cursor);
                }
                AvfxData::IsCameraSpace => {
                    avfx.is_camera_space = read_bool(&mut cursor);
                }
                AvfxData::IsFullEnvLight => {
                    avfx.is_full_env_light = read_bool(&mut cursor);
                }
                AvfxData::IsClipOwnSetting => {
                    avfx.is_clip_own_setting = read_bool(&mut cursor);
                }
                AvfxData::NearClipBegin => {
                    avfx.near_clip_begin = read_float(&mut cursor);
                }
                AvfxData::NearClipEnd => {
                    avfx.near_clip_end = read_float(&mut cursor);
                }
                AvfxData::FarClipBegin => {
                    avfx.far_clip_begin = read_float(&mut cursor);
                }
                AvfxData::FarClipEnd => {
                    avfx.far_clip_end = read_float(&mut cursor);
                }
                AvfxData::SoftParticleFadeRange => {
                    avfx.soft_particle_fade_range = read_float(&mut cursor);
                }
                AvfxData::SoftKeyOffset => {
                    avfx.soft_key_offset = read_float(&mut cursor);
                }
                AvfxData::DrawLayerType => {
                    avfx.draw_layer_type = read_uint(&mut cursor);
                }
                AvfxData::DrawOrderType => {
                    avfx.draw_order_type = read_uint(&mut cursor);
                }
                AvfxData::DirectionalLightSourceType => {
                    avfx.directional_light_source_type = read_uint(&mut cursor);
                }
                AvfxData::PointLightsType1 => {
                    avfx.point_lights_type1 = read_uint(&mut cursor);
                }
                AvfxData::PointLightsType2 => {
                    avfx.point_lights_type2 = read_uint(&mut cursor);
                }
                AvfxData::RevisedValuesPosX => {
                    avfx.revised_values_position[0] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesPosY => {
                    avfx.revised_values_position[1] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesPosZ => {
                    avfx.revised_values_position[2] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesRotX => {
                    avfx.revised_values_rotation[0] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesRotY => {
                    avfx.revised_values_rotation[1] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesRotZ => {
                    avfx.revised_values_rotation[2] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesScaleX => {
                    avfx.revised_values_scale[0] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesScaleY => {
                    avfx.revised_values_scale[1] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesScaleZ => {
                    avfx.revised_values_scale[2] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesColorR => {
                    avfx.revised_values_color[0] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesColorG => {
                    avfx.revised_values_color[1] = read_float(&mut cursor);
                }
                AvfxData::RevisedValuesColorB => {
                    avfx.revised_values_color[2] = read_float(&mut cursor);
                }
                AvfxData::FadeEnabledX => {
                    avfx.fade_enabled_x = read_bool(&mut cursor);
                }
                AvfxData::FadeInnerX => {
                    avfx.fade_inner[0] = read_float(&mut cursor);
                }
                AvfxData::FadeOuterX => {
                    avfx.fade_outer[0] = read_float(&mut cursor);
                }
                AvfxData::FadeEnabledY => {
                    avfx.fade_enabled_y = read_bool(&mut cursor);
                }
                AvfxData::FadeInnerY => {
                    avfx.fade_inner[1] = read_float(&mut cursor);
                }
                AvfxData::FadeOuterY => {
                    avfx.fade_outer[1] = read_float(&mut cursor);
                }
                AvfxData::FadeEnabledZ => {
                    avfx.fade_enabled_z = read_bool(&mut cursor);
                }
                AvfxData::FadeInnerZ => {
                    avfx.fade_inner[2] = read_float(&mut cursor);
                }
                AvfxData::FadeOuterZ => {
                    avfx.fade_outer[2] = read_float(&mut cursor);
                }
                AvfxData::GlobalFogEnabled => {
                    avfx.global_fog_enabled = read_bool(&mut cursor);
                }
                AvfxData::GlobalFogInfluence => {
                    avfx.global_fog_influence = read_float(&mut cursor);
                }
                AvfxData::LtsEnabled => {
                    avfx.lts_enabled = read_bool(&mut cursor);
                }
                AvfxData::AgsEnabled => {
                    avfx.ags_enabled = read_bool(&mut cursor);
                }
                AvfxData::NumSchedulers => {
                    todo!()
                }
                AvfxData::NumTimelines => {
                    todo!()
                }
                AvfxData::NumEmitters => {
                    todo!()
                }
                AvfxData::NumParticles => {
                    todo!()
                }
                AvfxData::NumEffectors => {
                    todo!()
                }
                AvfxData::NumBinders => {
                    todo!()
                }
                AvfxData::NumTextures => {
                    todo!()
                }
                AvfxData::NumModels => {
                    todo!()
                }
                AvfxData::Scheduler => {
                    todo!()
                }
                AvfxData::Timeline => {
                    todo!()
                }
                AvfxData::Emitter => {
                    todo!()
                }
                AvfxData::Particle => {
                    todo!()
                }
                AvfxData::Effector => {
                    todo!()
                }
                AvfxData::Binder => {
                    todo!()
                }
                AvfxData::Texture => {
                    todo!()
                }
                AvfxData::Model => {
                    todo!()
                }
                AvfxData::Unknown() => {}
            }
            let new_pos = cursor.position();
            let read_bytes = (new_pos - last_pos) - 8;
            let padding = block.size as u64 - read_bytes;
            cursor.seek(SeekFrom::Current(padding as i64)).ok()?;
        }

        Some(avfx)
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
        Avfx::from_existing(&read(d).unwrap());
    }
}
