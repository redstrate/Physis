// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use binrw::BinRead;
use binrw::binrw;
use crate::ByteSpan;
use crate::chardat::CharacterData;

#[binrw]
#[br(little)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RacialScalingParameters {
    /// Male minimum height
    pub male_min_size: f32,
    /// Male maximum height
    pub male_max_size: f32,

    /// Male minimum tail size
    pub male_min_tail: f32,
    /// Male maximum tail size
    pub male_max_tail: f32,

    /// Female minimum height
    pub female_min_size: f32,
    /// Female maximum height
    pub female_max_size: f32,

    /// Female minimum tail size
    pub female_min_tail: f32,
    /// Female maximum tail size
    pub female_max_tail: f32,

    /// Minimum bust size on the X-axis
    pub bust_min_x: f32,
    /// Minimum bust size on the Y-axis
    pub bust_min_y: f32,
    /// Minimum bust size on the Z-axis
    pub bust_min_z: f32,

    /// Maximum bust size on the X-axis
    pub bust_max_x: f32,
    /// Maximum bust size on the Y-axis
    pub bust_max_y: f32,
    /// Maximum bust size on the Z-axis
    pub bust_max_z: f32
}

#[derive(Debug)]
pub struct CMP {
    /// The racial scaling parameters
    pub parameters: Vec<RacialScalingParameters>
}

impl CMP {
    /// Parses an existing CMP file.
    pub fn from_existing(buffer: ByteSpan) -> Option<CMP> {
        let mut cursor = Cursor::new(buffer);

        cursor.seek(SeekFrom::Start(0x2a800)).ok()?;

        let rem = buffer.len() - cursor.position() as usize;
        let entries = rem / std::mem::size_of::<RacialScalingParameters>();

        let mut parameters = vec![];

        for _ in 0..entries {
            parameters.push(RacialScalingParameters::read(&mut cursor).ok()?);
        }

        Some(CMP {
            parameters
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        CMP::from_existing(&read(d).unwrap());
    }
}
