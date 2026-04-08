// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

#[binrw]
#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorHDRI {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
    pub intensity: f32,
}

#[binrw]
#[derive(Debug, PartialEq)]
pub struct GameInstanceObject {
    /// For IDs >= 1000000, index into the ENpcBase
    /// For IDs >= 2000000, the row ID reference to EObj/EObjName
    pub base_id: u32,
}
