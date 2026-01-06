// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

#[binrw]
#[derive(Debug, PartialEq)]
pub struct PathControlPoint {
    pub position: [f32; 3],
    pub point_id: u16,
    pub select: u8,
    pub _padding: u8,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct PathInstanceObject {
    pub control_points_unk: i32,
    #[br(temp)]
    #[bw(calc = control_points.len() as i32)]
    control_point_count: i32,
    _padding: [u32; 2],
    #[br(count = control_point_count)]
    pub control_points: Vec<PathControlPoint>,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ClientPathInstanceObject {
    pub parent_data: PathInstanceObject,
    pub ring: u8,
    _padding: u8,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct ServerPathInstanceObject {}
