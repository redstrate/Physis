// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::layer::Color;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ColorParameters {
    pub eyes: [Color; 256],
    pub hair_highlights: [Color; 256],
    pub lips_dark: [Color; 128],
    pub face_paint_dark: [Color; 128],
    pub features: [Color; 256],
    pub lips_light: [Color; 128],
    pub face_paint_light: [Color; 128],
    pub unused_eyes1: [Color; 256],
    pub unused_eyes2: [Color; 256],
    pub unused_eyes3: [Color; 256],
    pub unused_features: [Color; 256],
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HairColor {
    pub main: Color,
    pub unused_sheen: Color,
}

#[binrw]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GenderClanColorParameters {
    pub skin: [Color; 256],
    pub hair: [HairColor; 256],
    pub skin_interface: [Color; 256],
    pub hair_interface: [Color; 256],
}

/// A set of scaling parameters for a race.
#[binrw]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RacialScalingParameters {
    /// Male minimum height.
    pub male_min_size: f32,
    /// Male maximum height.
    pub male_max_size: f32,

    /// Male minimum tail size.
    pub male_min_tail: f32,
    /// Male maximum tail size.
    pub male_max_tail: f32,

    /// Female minimum height.
    pub female_min_size: f32,
    /// Female maximum height.
    pub female_max_size: f32,

    /// Female minimum tail size.
    pub female_min_tail: f32,
    /// Female maximum tail size.
    pub female_max_tail: f32,

    /// Minimum bust size on the X-axis.
    pub bust_min_x: f32,
    /// Minimum bust size on the Y-axis.
    pub bust_min_y: f32,
    /// Minimum bust size on the Z-axis.
    pub bust_min_z: f32,

    /// Maximum bust size on the X-axis.
    pub bust_max_x: f32,
    /// Maximum bust size on the Y-axis.
    pub bust_max_y: f32,
    /// Maximum bust size on the Z-axis.
    pub bust_max_z: f32,
}

/// Character multiplier make file, usually with the `.cmp` file extension.
///
/// This is used to determine various scaling limits for height, and so on.
#[binrw]
#[repr(C)]
#[derive(Debug)]
pub struct CMP {
    pub parameters: ColorParameters,
    pub interface: ColorParameters,
    pub races: [GenderClanColorParameters; 32],
    pub scales: [[RacialScalingParameters; 10]; 8],
}

impl ReadableFile for CMP {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);

        Ok(CMP::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

impl WritableFile for CMP {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_invalid() {
        // TODO: restore once it doesn't crash
        // pass_random_invalid::<CMP>();
    }
}
