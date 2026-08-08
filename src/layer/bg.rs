// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::{
    common_file_operations::write_bool_as,
    layer::collision::{
        CollisionAttributes, read_collision_attributes, write_collision_attributes,
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
    pub unk_offset: i32, // TODO: probably some sort of collision config?
    /// Controls whether the render model is visible.
    /// Unknown effect on the collision mesh.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub visible: bool,
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
            unk_offset: Default::default(),
            visible: true,
            world_light_shadow_mode: Default::default(),
            object_light_shadow_mode: Default::default(),
            fade_out_distance: Default::default(),
            bounding_sphere_size: Default::default(),
        }
    }
}
