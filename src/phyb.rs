// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use crate::common_file_operations::read_string_until_null;
use crate::common_file_operations::write_string;
use binrw::BinRead;
use binrw::binrw;
use bitflags::bitflags;

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybHeader {
    /// Only seen 16777217 (0x1000001) so far.
    version: i32,
    data_type: u32,
    /// Offset to bone collision data.
    collision_offset: u32,
    /// Offset to bone simulator data.
    simulator_offset: u32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybCapsule {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub start_bone_name: String,
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub end_bone_name: String,
    pub start_offset: [f32; 3],
    pub end_offset: [f32; 3],
    pub radius: f32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybEllipsoid {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub bone_name: String,
    offset1: [f32; 3],
    offset2: [f32; 3],
    offset3: [f32; 3],
    offset4: [f32; 3],
    pub radius: f32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybNormalPlane {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub bone_name: String,
    pub bone_offset: [f32; 3],
    pub normal: [f32; 3],
    pub thickness: f32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybThreePointPlane {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub bone_name: String,
    unk1: [f32; 4],
    unk2: [f32; 4],
    unk3: [f32; 4],
    unk4: [f32; 4],
    pub bone_offset: [f32; 3],
    unk5: [f32; 3],
    unk6: [f32; 3],
    pub thickness: f32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybSphere {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub bone_name: String,
    pub bone_offset: [f32; 3],
    pub thickness: f32,
}

/// Physics collision data.
#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybCollision {
    num_capsules: u8,
    num_ellipsoids: u8,
    num_normal_planes: u8,
    num_three_point_planes: u8,
    num_spheres: u8,
    unk1: [u8; 3], // FIXME: VFXEditor says this is padding

    #[br(count = num_capsules)]
    pub capsules: Vec<PhybCapsule>,

    #[br(count = num_ellipsoids)]
    pub ellipsoids: Vec<PhybEllipsoid>,

    #[br(count = num_normal_planes)]
    pub normal_planes: Vec<PhybNormalPlane>,

    #[br(count = num_three_point_planes)]
    pub three_point_planes: Vec<PhybThreePointPlane>,

    #[br(count = num_spheres)]
    pub spheres: Vec<PhybSphere>,
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SimulatorFlags(u8);

bitflags! {
    impl SimulatorFlags : u8 {
        const SIMULATING = 0x01;
        const COLLISIONS_HANDLED = 0x02;
        const CONTINUOUS_COLLISIONS = 0x04;
        const USING_GROUND_PLANE = 0x08;
        const FIXED_LENGTH = 0x10;
    }
}

#[binrw]
#[brw(repr = i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum CollisionType {
    Both = 0,
    Outside = 1,
    Inside = 2,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybCollisionData {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub name: String,
    pub collision_type: CollisionType,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
#[br(import(header: &PhybHeader))]
pub struct PhybNode {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub bone_name: String,
    pub radius: f32,
    pub attract_by_animation: f32,
    pub wind_scale: f32,
    pub gravy_scale: f32,
    pub cone_max_angle: f32,
    pub conve_axis_offset: [f32; 3],
    pub constraint_plane_normal: [f32; 3],
    pub collision_flags: u32,
    pub continuous_collision_flags: u32,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub enum ChainType {
    Sphere = 0,
    Capsule = 1,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
#[br(import(header: &PhybHeader))]
pub struct PhybChain {
    num_collisions: u16,
    num_nodes: u16,

    pub dampening: f32,
    pub max_speed: f32,
    pub friction: f32,
    pub collision_dapening: f32,
    pub repulsion_strength: f32,
    pub last_bone_offset: [f32; 3],
    pub chain_type: ChainType,

    collision_offset: u32,
    node_offset: u32,

    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + collision_offset as u64 + 4))]
    #[br(count = num_collisions, restore_position)]
    pub collisions: Vec<PhybCollisionData>,

    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + node_offset as u64 + 4))]
    #[br(count = num_nodes, args { inner: (header,) }, restore_position)]
    pub nodes: Vec<PhybNode>,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybConnector {
    pub chain_id_1: u16,
    pub chain_id_2: u16,
    pub node_id_1: u16,
    pub node_id_2: u16,
    pub collision_radius: f32,
    pub friction: f32,
    pub dampening: f32,
    pub repulsion: f32,
    pub collision_flags: u32,
    pub continuous_collision_flags: u32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybAttract {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub bone_name: String,
    pub bone_offset: [f32; 3],
    pub chain_id: u16,
    pub node_id: u16,
    pub stiffness: f32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybPin {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub bone_name: String,
    pub bone_offset: [f32; 3],
    pub chain_id: u16,
    pub node_id: u16,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybSpring {
    pub chain_id_1: u16,
    pub chain_id_2: u16,
    pub node_id_1: u16,
    pub node_id_2: u16,
    pub stretch_stiffness: f32,
    pub compress_stiffness: f32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybPostAlignment {
    #[brw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(parse_with = read_string_until_null)]
    pub collision_name: String,
    pub chain_id: u16,
    pub node_id: u16,
}

/// Physics simulator data.
#[binrw]
#[br(import(header: &PhybHeader))]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PhybSimulator {
    num_collision_data: u8,
    num_collision_connector: u8,
    num_chain: u8,
    num_connector: u8,
    num_attract: u8,
    num_pin: u8,
    num_spring: u8,
    num_post_alignment: u8,

    /// Initial gravity for this simulator.
    pub gravity: [f32; 3],
    /// Initial wind for this simulator.
    pub wind: [f32; 3],
    constraint_loop: u16,
    collision_loop: u16,
    /// The flags for this simulator.
    pub flags: SimulatorFlags,
    /// Index into the PhysicsGroup Excel sheet.
    pub group: u8,
    // TODO: VFX Editor says the next two bytes are padding
    unk1: [u8; 2],

    collision_object_offset: u32,
    collision_connector_offset: u32,
    chain_offset: u32,
    connector_offset: u32,
    attract_offset: u32,
    pin_offset: u32,
    spring_offset: u32,
    post_alignment_offset: u32,

    /// Collision objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + collision_object_offset as u64 + 4))]
    #[br(count = num_collision_data, restore_position)]
    pub collision_objects: Vec<PhybCollisionData>,

    /// Collision connection objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + collision_connector_offset as u64 + 4))]
    #[br(count = num_collision_connector, restore_position)]
    pub collision_connections: Vec<PhybCollisionData>,

    /// Chain objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + chain_offset as u64 + 4))]
    #[br(count = num_chain, args { inner: (header,) }, restore_position)]
    pub chains: Vec<PhybChain>,

    /// Connector objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + connector_offset as u64 + 4))]
    #[br(count = num_connector, restore_position)]
    pub connectors: Vec<PhybConnector>,

    /// Attract objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + attract_offset as u64 + 4))]
    #[br(count = num_attract, restore_position)]
    pub attracts: Vec<PhybAttract>,

    /// Pin objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + pin_offset as u64 + 4))]
    #[br(count = num_pin, restore_position)]
    pub pins: Vec<PhybPin>,

    /// Spring objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + spring_offset as u64 + 4))]
    #[br(count = num_spring, restore_position)]
    pub springs: Vec<PhybSpring>,

    /// Post alignment objects for this simulator.
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64 + post_alignment_offset as u64 + 4))]
    #[br(count = num_post_alignment, restore_position)]
    pub post_aligmments: Vec<PhybPostAlignment>,
}

/// Physics binary file, usually with the `.phyb` file extension.
///
/// Contains information to set up bone simulators and colliders.
#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct Phyb {
    header: PhybHeader,
    /// The contained collision data.
    #[br(if(header.collision_offset != header.simulator_offset), seek_before = SeekFrom::Start(header.collision_offset as u64))]
    pub collision: Option<PhybCollision>,
    #[br(seek_before = SeekFrom::Start(header.simulator_offset as u64))]
    #[bw(calc = simulators.len() as u32)]
    #[br(temp)]
    num_simulators: u32,
    /// The contained simulator data.
    #[br(count = num_simulators, args { inner: (&header,) })]
    pub simulators: Vec<PhybSimulator>,
}

impl ReadableFile for Phyb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Phyb::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Phyb>();
    }
}
