// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use binrw::BinRead;
use binrw::binrw;
use crate::ByteSpan;

#[binrw]
#[derive(Debug, Clone, Copy)]
#[brw(little)]
struct PlatePosition {
    x: i16,
    y: i16
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct TerrainHeader {
    version: u32,
    plate_count: u32,
    plate_size: u32,
    clip_distance: f32,

    unknown: f32,

    #[br(count = 32)]
    padding: Vec<u8>,

    #[br(count = plate_count)]
    positions: Vec<PlatePosition>
}

#[derive(Debug)]
pub struct PlateModel {
    pub position: (f32, f32),
    pub filename: String
}

#[derive(Debug)]
pub struct Terrain {
    pub plates: Vec<PlateModel>
}

impl Terrain {
    /// Reads an existing TERA file
    pub fn from_existing(buffer: ByteSpan) -> Option<Terrain> {
        let mut cursor = Cursor::new(buffer);
        let header = TerrainHeader::read(&mut cursor).ok()?;

        let mut plates = vec![];

        for i in 0..header.plate_count {
            plates.push(PlateModel {
                position: (header.plate_size as f32 * (header.positions[i as usize].x as f32 + 0.5),
                           header.plate_size as f32 * (header.positions[i as usize].y as f32 + 0.5)),
                filename: format!("{:04}.mdl", i)
            })
        }

        Some(Terrain {
            plates
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
        Terrain::from_existing(&read(d).unwrap());
    }
}
