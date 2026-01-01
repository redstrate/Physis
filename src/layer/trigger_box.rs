// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::common_file_operations::write_bool_as;

use super::read_bool_from;

#[binrw]
#[repr(C)]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TriggerBoxShape {
    None = 0x0,
    Box = 0x1,
    Sphere = 0x2,
    Cylinder = 0x3,
    Plane = 0x4,
    Mesh = 0x5,
    PlaneTwoSided = 0x6,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct TriggerBoxInstanceObject {
    pub trigger_box_shape: TriggerBoxShape,
    pub priority: i16,
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    #[brw(pad_after = 5)] // padding
    pub enabled: bool,
}
