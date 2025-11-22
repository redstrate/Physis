// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Copy)]
#[brw(little)]
struct PlatePosition {
    x: i16,
    y: i16,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct TerrainHeader {
    // Example: 0x1000003
    version: u32,
    plate_count: u32,
    plate_size: u32,
    clip_distance: f32,

    unknown: f32,

    #[brw(pad_before = 32)]
    #[br(count = plate_count)]
    positions: Vec<PlatePosition>,
}

#[derive(Debug)]
pub struct PlateModel {
    pub position: (f32, f32),
    pub filename: String,
}

/// Terrain file, usually with the `.tera` file extension.
///
/// Contains a list of plates that make up the underlying terrain of a level.
#[derive(Debug)]
pub struct Terrain {
    pub plates: Vec<PlateModel>,
}

impl Terrain {
    /// Reads an existing TERA file
    pub fn from_existing(buffer: ByteSpan) -> Option<Terrain> {
        let mut cursor = Cursor::new(buffer);
        let header = TerrainHeader::read(&mut cursor).ok()?;

        let mut plates = vec![];

        for i in 0..header.plate_count {
            plates.push(PlateModel {
                position: (
                    header.plate_size as f32 * (header.positions[i as usize].x as f32 + 0.5),
                    header.plate_size as f32 * (header.positions[i as usize].y as f32 + 0.5),
                ),
                filename: format!("{:04}.mdl", i),
            })
        }

        Some(Terrain { plates })
    }

    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);

            let plate_size = 128;

            let header = TerrainHeader {
                version: 0x1000003,
                plate_count: self.plates.len() as u32,
                plate_size,
                clip_distance: 0.0, // TODO: make configurable
                unknown: 1.0,       // TODO: what is this
                positions: self
                    .plates
                    .iter()
                    .map(|model| PlatePosition {
                        x: ((model.position.0 / plate_size as f32) - 0.5) as i16,
                        y: ((model.position.1 / plate_size as f32) - 0.5) as i16,
                    })
                    .collect(),
            };
            header.write_le(&mut cursor).ok()?;
        }

        Some(buffer)
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
