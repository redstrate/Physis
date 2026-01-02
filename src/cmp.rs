// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::binrw;

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
#[derive(Debug)]
pub struct CMP {
    /// The racial scaling parameters
    pub parameters: Vec<RacialScalingParameters>,
}

impl ReadableFile for CMP {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);

        cursor.seek(SeekFrom::Start(0x2a800)).ok()?;

        let rem = buffer.len() - cursor.position() as usize;
        let entries = rem / std::mem::size_of::<RacialScalingParameters>();

        let mut parameters = vec![];

        for _ in 0..entries {
            parameters.push(
                RacialScalingParameters::read_options(&mut cursor, platform.endianness(), ())
                    .ok()?,
            );
        }

        Some(CMP { parameters })
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<CMP>();
    }
}
