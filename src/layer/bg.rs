// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::binrw;

use crate::{
    AABB,
    common_file_operations::write_bool_as,
    layer::{
        Transformation,
        collision::{CollisionAttributes, read_collision_attributes, write_collision_attributes},
    },
    string_heap::{HeapPointer, HeapString},
};

use super::{StringHeap, read_bool_from};

#[binrw]
#[repr(i32)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ModelCollisionType {
    /// Don't use collision from the PCB, if provided.
    #[default]
    None = 0x0,
    /// Use collision from the PCB.
    Replace = 0x1,
    /// Unknown purpose.
    Box = 0x2,
}

#[binrw]
#[repr(u8)]
#[brw(repr = u8)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum ShadowMode {
    /// Forces shadows off.
    ForceOff = 0x0,
    /// Forces shadows on.
    ForceOn = 0x1,
    /// Inherits the shadow mode from... somewhere - the model?
    #[default]
    Inherit = 0x2,
}

/// Background model object that can have collision.
#[binrw]
#[derive(Debug, PartialEq, Clone)]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub struct BgPartInstanceObject {
    /// Path to a `.mdl` for the visual model.
    #[brw(args(heap_pointer, string_heap))]
    pub asset_path: HeapString,
    /// Path to a `.pcb` for the collision model.
    #[brw(args(heap_pointer, string_heap))]
    pub collision_asset_path: HeapString,
    /// How collision for this object be handled.
    pub collision_type: ModelCollisionType,
    #[br(parse_with = read_collision_attributes)]
    #[bw(write_with = write_collision_attributes)]
    pub collision_attributes: CollisionAttributes,
    #[br(temp)]
    #[bw(calc = 0)]
    offset_collider_analytic_data: i32,
    #[br(if(offset_collider_analytic_data != 0), restore_position, seek_before = SeekFrom::Current(offset_collider_analytic_data as i64 - 80))]
    pub analytic_collider: Option<AnalyticCollider>,
    /// Controls whether the render model is visible.
    /// Unknown effect on the collision mesh.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub visible: bool, // NOTE: I'm not sure why, but something in retail puts 255 here. That's still considered "true" but that's strange.
    /// Controls whether the visual model will cast shadows from the world light.
    /// Has no effect if the model has no shadow mesh?
    #[br(try)]
    pub world_light_shadow_mode: ShadowMode,
    /// Controls whether the visual model will cast shadows from normal Light objects.
    #[brw(pad_after = 1)] // padding, not read
    #[br(try)]
    pub object_light_shadow_mode: ShadowMode,
    /// Distance between the camera and this object before it fades out of existence.
    /// Currently only has an effect if the LOD option is turned on.
    /// If set to zero, it means the object doesn't want to override what's set by the visual model.
    pub fade_out_distance: f32,
    /// Unknown purpose, but used in BgPartsLayoutInstance.GetBoundingSphere in some way.
    pub bounding_sphere_size: f32,
}

impl Default for BgPartInstanceObject {
    fn default() -> Self {
        Self {
            asset_path: Default::default(),
            collision_asset_path: Default::default(),
            collision_type: Default::default(),
            collision_attributes: Default::default(),
            analytic_collider: None,
            visible: true,
            world_light_shadow_mode: Default::default(),
            object_light_shadow_mode: Default::default(),
            fade_out_distance: Default::default(),
            bounding_sphere_size: Default::default(),
        }
    }
}

#[binrw]
#[derive(Debug, PartialEq, Clone)]
pub struct AnalyticCollider {
    pub material_mask: u32,
    pub material_id: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub collider_type: AnalyticColliderType,
    pub transform: Transformation,
    pub bounds: AABB,
}

#[binrw]
#[repr(u32)]
#[brw(repr = u32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum AnalyticColliderType {
    #[default]
    None,
    Box,
    Sphere,
    Cylinder,
    Plane,
}
