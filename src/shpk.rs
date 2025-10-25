// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::{Cursor, SeekFrom};

use crate::crc::XivCrc32;
use crate::{ByteSpan, common_file_operations::read_bool_from};
use binrw::{BinRead, binread};

#[binread]
#[br(little, import {
    strings_offset: u32
})]
#[derive(Debug)]
#[allow(unused)]
pub struct ResourceParameter {
    /// CRC32 of `name`.
    crc32: u32,
    #[br(temp)]
    local_string_offset: u32,
    #[br(temp)]
    string_length: u16,

    // Seems to only be 0x1 for textures.
    unknown1: u16,

    // May be a flag of some sort?
    unknown2: u16,
    /// In vec4s, e.g. a size of 1 means its one vec4.
    pub size: u16,

    #[br(seek_before = SeekFrom::Start(strings_offset as u64 + local_string_offset as u64))]
    #[br(count = string_length, map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    #[br(restore_position)]
    pub name: String,
}

#[binread]
#[br(little, import {
    version: u32,
    shader_data_offset: u32,
    strings_offset: u32,
    is_vertex: bool
})]
#[derive(Debug)]
#[allow(unused)]
pub struct Shader {
    data_offset: u32,
    data_size: u32,

    scalar_parameter_count: u16,
    resource_parameter_count: u16,
    uav_parameter_count: u16,
    texture_count: u16,

    #[br(if(version >= 0x0D01))]
    unk1: Option<u16>,
    #[br(if(version >= 0x0D01))]
    unk2: Option<u16>,

    #[br(args { count: scalar_parameter_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    pub scalar_parameters: Vec<ResourceParameter>,
    #[br(args { count: resource_parameter_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    pub resource_parameters: Vec<ResourceParameter>,
    #[br(args { count: uav_parameter_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    pub uav_parameters: Vec<ResourceParameter>,
    #[br(args { count: texture_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    pub texture_parameters: Vec<ResourceParameter>,

    /// The HLSL bytecode of this shader. The DX level used varies.
    // TODO: dx9 is 4, not 8
    #[br(seek_before = SeekFrom::Start(shader_data_offset as u64 + data_offset as u64 + if is_vertex { 8 } else { 0 }))]
    #[br(count = data_size)]
    #[br(restore_position)]
    pub bytecode: Vec<u8>,
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(unused)]
pub struct MaterialParameter {
    id: u32,
    byte_offset: u16,
    byte_size: u16,
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[allow(unused)]
pub struct Key {
    /// CRC32 of the key.
    pub id: u32,
    /// Default value you should provide (also a CRC32) if the material doesn't have this `id`.
    pub default_value: u32,
}

#[binread]
#[repr(C)]
#[br(little, import {
    version: u32,
})]
#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub struct Pass {
    id: u32,
    vertex_shader: u32,
    #[brw(pad_after = if version >= 0x0D01 { 12 } else { 0 })]
    pixel_shader: u32,
}

#[binread]
#[derive(Debug)]
#[allow(unused)]
pub struct NodeAlias {
    selector: u32,
    node: u32,
}

#[binread]
#[br(little, import {
    version: u32,
    system_key_count: u32,
    scene_key_count: u32,
    material_key_count: u32,
    subview_key_count: u32
})]
#[derive(Debug)]
#[allow(unused)]
pub struct Node {
    pub selector: u32,
    pub pass_count: u32,
    pub pass_indices: [u8; 16],

    #[br(count = if version >= 0x0D01 { 8 } else { 0 } )]
    unk_data: Vec<u8>,

    #[br(count = system_key_count)]
    pub system_keys: Vec<u32>,
    #[br(count = scene_key_count)]
    pub scene_keys: Vec<u32>,
    #[br(count = material_key_count)]
    pub material_keys: Vec<u32>,
    #[br(count = subview_key_count)]
    pub subview_keys: Vec<u32>,
    #[br(args { count: pass_count as usize, inner: PassBinReadArgs { version }})]
    pub passes: Vec<Pass>,
}

#[binread]
#[br(little)]
#[br(magic = b"ShPk")]
#[derive(Debug)]
#[allow(dead_code, unused)]
pub struct ShaderPackage {
    /// For example, 3329
    version: u32,

    // "DX9\0" or "DX11"
    #[br(count = 4)]
    #[bw(pad_size_to = 4)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    format: String,

    file_length: u32,
    shader_data_offset: u32,
    strings_offset: u32,

    vertex_shader_count: u32,
    pixel_shader_count: u32,

    pub material_parameters_size: u32,
    material_parameter_count: u16,

    #[br(map = read_bool_from::<u16>)]
    has_mat_param_defaults: bool,
    scalar_parameter_count: u32,
    sampler_count: u16,
    texture_count: u16,
    uav_count: u32,

    system_key_count: u32,
    scene_key_count: u32,
    material_key_count: u32,
    node_count: u32,
    node_alias_count: u32,

    #[br(count = if version >= 0x0D01 { 12 } else { 0 } )]
    unk_data: Vec<u8>,

    // TODO: dx9 needs 4 bytes of padding, dx11 is 8 (correct)
    #[br(args { count: vertex_shader_count as usize, inner : ShaderBinReadArgs { version, is_vertex: true, shader_data_offset, strings_offset }})]
    pub vertex_shaders: Vec<Shader>,
    #[br(args { count: pixel_shader_count as usize, inner: ShaderBinReadArgs { version, is_vertex: false, shader_data_offset, strings_offset } })]
    pub pixel_shaders: Vec<Shader>,

    #[br(count = material_parameter_count)]
    pub material_parameters: Vec<MaterialParameter>,

    #[br(count = if has_mat_param_defaults { (material_parameters_size as i32) >> 2i32 } else { 0 })]
    pub mat_param_defaults: Vec<f32>,

    #[br(args { count: scalar_parameter_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    scalar_parameters: Vec<ResourceParameter>,
    #[br(args { count: sampler_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    sampler_parameters: Vec<ResourceParameter>,
    #[br(args { count: texture_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    texture_parameters: Vec<ResourceParameter>,
    #[br(args { count: uav_count as usize, inner: ResourceParameterBinReadArgs { strings_offset }})]
    uav_parameters: Vec<ResourceParameter>,

    #[br(count = system_key_count)]
    pub system_keys: Vec<Key>,
    #[br(count = scene_key_count)]
    pub scene_keys: Vec<Key>,
    #[br(count = material_key_count)]
    pub material_keys: Vec<Key>,

    /// CRC32 of the default SUB_VIEW key.
    pub sub_view_key1_default: u32,
    /// CRC32 of the default SUB_VIEW key.
    pub sub_view_key2_default: u32,

    #[br(args { count: node_count as usize, inner: NodeBinReadArgs { version, system_key_count, scene_key_count, material_key_count, subview_key_count: 2 }})]
    pub nodes: Vec<Node>,

    #[br(ignore)]
    node_selectors: Vec<(u32, u32)>,

    #[br(count = node_alias_count)]
    node_aliases: Vec<NodeAlias>,
}

const SELECTOR_MULTIPLER: u32 = 31;

impl ShaderPackage {
    /// Reads an existing SHPK file
    pub fn from_existing(buffer: ByteSpan) -> Option<ShaderPackage> {
        let mut cursor = Cursor::new(buffer);
        let mut package = ShaderPackage::read(&mut cursor).ok()?;

        for (i, node) in package.nodes.iter().enumerate() {
            package.node_selectors.push((node.selector, i as u32));
        }
        for alias in &package.node_aliases {
            package.node_selectors.push((alias.selector, alias.node));
        }

        Some(package)
    }

    pub fn find_node(&self, selector: u32) -> Option<&Node> {
        for (sel, node) in &self.node_selectors {
            if *sel == selector {
                return Some(&self.nodes[*node as usize]);
            }
        }

        None
    }

    pub fn build_selector_from_all_keys(
        system_keys: &[u32],
        scene_keys: &[u32],
        material_keys: &[u32],
        subview_keys: &[u32],
    ) -> u32 {
        Self::build_selector_from_keys(
            Self::build_selector(system_keys),
            Self::build_selector(scene_keys),
            Self::build_selector(material_keys),
            Self::build_selector(subview_keys),
        )
    }

    pub fn build_selector_from_keys(
        system_key: u32,
        scene_key: u32,
        material_key: u32,
        subview_key: u32,
    ) -> u32 {
        Self::build_selector(&[system_key, scene_key, material_key, subview_key])
    }

    pub fn build_selector(keys: &[u32]) -> u32 {
        let mut selector: u32 = 0;
        let mut multiplier: u32 = 1;

        for key in keys {
            selector = selector.wrapping_add(key.wrapping_mul(multiplier));
            multiplier = multiplier.wrapping_mul(SELECTOR_MULTIPLER);
        }

        selector
    }

    pub fn crc(str: &str) -> u32 {
        XivCrc32::from(str).crc
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
        ShaderPackage::from_existing(&read(d).unwrap());
    }

    #[test]
    fn test_crc() {
        assert_eq!(ShaderPackage::crc("PASS_0"), 0xC5A5389C);
        assert_eq!(ShaderPackage::crc("DecodeDepthBuffer"), 0x2C6C023C);
        assert_eq!(ShaderPackage::crc("g_Sampler"), 0x88408C04);
    }

    #[test]
    fn test_selector() {
        let selector = ShaderPackage::build_selector_from_all_keys(
            &[],
            &[
                ShaderPackage::crc("TransformViewSkin"),
                ShaderPackage::crc("GetAmbientLight_SH"),
                ShaderPackage::crc("GetReflectColor_Texture"),
                ShaderPackage::crc("GetAmbientOcclusion_None"),
                ShaderPackage::crc("ApplyDitherClipOff"),
            ],
            &[3756477356, 1556481461, 1111668802, 428675533],
            &[
                ShaderPackage::crc("Default"),
                ShaderPackage::crc("SUB_VIEW_MAIN"),
            ],
        );

        assert_eq!(selector, 0x1075AE91);
    }
}
