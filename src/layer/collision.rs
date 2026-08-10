// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{BinRead, BinResult, BinWriterExt, Endian, binrw};

use crate::{
    common_file_operations::{read_bool_from, write_bool_as},
    string_heap::{HeapPointer, HeapString, StringHeap},
};

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct CollisionAttributes {
    pub material_id: u64,
    pub material_mask: u64,
}

#[binrw]
struct CollisionAttributesNormal {
    #[br(temp)]
    #[bw(calc = *collision_material_mask as u32)]
    collision_material_mask_low: u32,
    #[br(temp)]
    #[bw(calc = *collision_material_id as u32)]
    collision_material_id_low: u32,
    #[br(temp)]
    #[bw(calc = (*collision_material_mask >> 32) as u32)]
    collision_material_mask_high: u32,
    #[br(temp)]
    #[bw(calc = (*collision_material_id >> 32) as u32)]
    collision_material_id_high: u32,
    #[br(calc = ((collision_material_id_high as u64) << 32) | collision_material_id_low as u64)]
    #[bw(ignore)] // written above
    collision_material_id: u64,
    #[br(calc = ((collision_material_mask_high as u64) << 32) | collision_material_mask_low as u64)]
    #[bw(ignore)] // written above
    collision_material_mask: u64,
}

// TODO: Is probably not a PS3-ism but something of the era, but I need to find a PC build of this version to confirm.
#[binrw]
struct CollisionAttributesPS3 {
    collision_material_mask: u32,
    collision_material_id: u32,
}

#[binrw::parser(reader, endian)]
pub(crate) fn read_collision_attributes() -> BinResult<CollisionAttributes> {
    match endian {
        Endian::Big => {
            let attributes = CollisionAttributesPS3::read_options(reader, endian, ())?;

            Ok(CollisionAttributes {
                material_id: attributes.collision_material_id as u64,
                material_mask: attributes.collision_material_mask as u64,
            })
        }
        Endian::Little => {
            let attributes = CollisionAttributesNormal::read_options(reader, endian, ())?;

            Ok(CollisionAttributes {
                material_id: attributes.collision_material_id,
                material_mask: attributes.collision_material_mask,
            })
        }
    }
}

#[binrw::writer(writer, endian)]
pub(crate) fn write_collision_attributes(attributes: &CollisionAttributes) -> BinResult<()> {
    match endian {
        Endian::Big => {
            let new_attributes = CollisionAttributesPS3 {
                collision_material_mask: attributes.material_mask as u32,
                collision_material_id: attributes.material_id as u32,
            };
            writer.write_type_args(&new_attributes, endian, ())?;
        }
        Endian::Little => {
            let new_attributes = CollisionAttributesNormal {
                collision_material_id: attributes.material_id,
                collision_material_mask: attributes.material_mask,
            };
            writer.write_type_args(&new_attributes, endian, ())?;
        }
    }
    Ok(())
}

/// Unknown object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[br(import(string_heap: &StringHeap, heap_pointer: HeapPointer))]
#[bw(import(string_heap: &mut StringHeap, heap_pointer: HeapPointer))]
pub struct CollisionBoxInstanceObject {
    pub parent_data: TriggerBoxInstanceObject,
    #[br(parse_with = read_collision_attributes)]
    #[bw(write_with = write_collision_attributes)]
    pub collision_attributes: CollisionAttributes,
    /// If true, CollisionBoxLayoutInstance.GetLayerMask returns 0x43. Otherwise returns 1.
    #[brw(pad_after = 3)] // Padding, not read
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub layer_mask_is_43h: bool,

    /// Path to the PCB if `trigger_box_shape` is `Mesh`.
    #[brw(args(heap_pointer, string_heap))]
    pub collision_asset_path: HeapString,
}

#[binrw]
#[repr(u32)]
#[brw(repr = u32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum TriggerBoxShape {
    #[default]
    None,
    Box,
    Sphere,
    Cylinder,
    Plane,
    Mesh,
    PlaneTwoSided,
}

/// Base struct for collision objects.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TriggerBoxInstanceObject {
    pub trigger_box_shape: TriggerBoxShape,
    pub priority: i16,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 5)] // Padding, not read
    pub enabled: bool,
}

/// Unknown object.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct CullingBoxInstanceObject {
    unk1: u32,
}
