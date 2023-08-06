// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use binrw::BinRead;
use binrw::binrw;

use crate::gamedata::MemoryBuffer;

#[binrw]
#[br(little)]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RacialScalingParameters {
    pub male_min_size: f32,
    pub male_max_size: f32,

    pub male_min_tail: f32,
    pub male_max_tail: f32,

    pub female_min_size: f32,
    pub female_max_size: f32,

    pub female_min_tail: f32,
    pub female_max_tail: f32,

    pub bust_min_x: f32,
    pub bust_min_y: f32,
    pub bust_min_z: f32,

    pub bust_max_x: f32,
    pub bust_max_y: f32,
    pub bust_max_z: f32
}

#[derive(Debug)]
pub struct CMP {
    pub parameters: Vec<RacialScalingParameters>
}

impl CMP {
    /// Parses an existing FIIN file.
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<CMP> {
        let mut cursor = Cursor::new(buffer);

        cursor.seek(SeekFrom::Start(0x2a800)).unwrap();

        let rem = buffer.len() - cursor.position() as usize;
        let entries = rem / std::mem::size_of::<RacialScalingParameters>();

        let mut parameters = vec![];

        for _ in 0..entries {
            parameters.push(RacialScalingParameters::read(&mut cursor).unwrap());
        }

        Some(CMP {
            parameters
        })
    }
}