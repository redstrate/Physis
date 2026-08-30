// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;

use binrw::binrw;

use crate::common_file_operations::{read_bool_from, write_bool_as};

#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PathControlPoint {
    pub position: [f32; 3],
    pub point_id: u16,
    #[brw(pad_after = 1)] // pading
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub select: bool,
}

/// Base struct for path objects.
#[binrw]
#[brw(import(size: i32))]
#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PathInstanceObject {
    #[br(temp)]
    #[bw(calc = 56 + size)]
    control_point_offset: i32,
    #[br(temp)]
    #[bw(calc = control_points.len() as i32)]
    control_point_count: i32,
    #[br(restore_position, seek_before = SeekFrom::Current(control_point_offset as i64 - 56), count = control_point_count)]
    pub control_points: Vec<PathControlPoint>,
}

/// Path object that objects and characters can follow.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ClientPathInstanceObject {
    #[brw(args(24))]
    pub parent_data: PathInstanceObject,
    #[brw(pad_after = 7)] // not read
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk1: bool,
    #[brw(pad_after = 1)] // not read
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk2: bool,
    #[brw(pad_after = 4)] // not read
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub unk3: bool,
    /// Offset to a vec4?
    pub unk_offset1: u32,
    /// Offset to a vec3?
    pub unk_offset2: u32,
}

/// Path object that objects and characters can follow.
///
/// This is stripped out of retail data, and is not used by the client.
#[binrw]
#[derive(Debug, PartialEq, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ServerPathInstanceObject {
    #[brw(args(0))]
    pub parent_data: PathInstanceObject,
}
