// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[derive(Debug, Clone, Copy)]
pub struct Plate {
    /// X position of this plate.
    pub x: i16,
    /// Y position of this plate.
    pub y: i16,
}

/// Terrain file, usually with the `.tera` file extension.
///
/// Contains a list of plates that make up the underlying terrain of a level.
#[binrw]
#[derive(Debug, Clone)]
pub struct Terrain {
    /// What version this file is.
    pub version: u32,
    /// How many plates are in this file.
    #[bw(calc = plates.len() as u32)]
    #[br(temp)]
    plate_count: u32,
    /// Size of each plate in units.
    pub plate_size: u32,
    /// Distance past which the terrain is not drawn, which is zero in most files.
    pub clip_distance: f32,
    /// How far a plate's textures blend into its neighbours, over `0.0..=1.0`.
    pub edge_bias: f32,
    /// Mask of the texture slots the plate materials sample with the alternate mip LOD bias, the colour slot in the lowest bit, then normal and specular. No other bit is ever set.bi
    #[brw(pad_after = 28)]
    pub sampler_bias: u32,
    /// The plates contained within this file,
    #[br(count = plate_count)]
    pub plates: Vec<Plate>,
}

impl Terrain {
    pub const VERSION_V1_0_0_1: u32 = 0x01000001;
    pub const VERSION_V1_0_0_2: u32 = 0x01000002;
    pub const VERSION_V1_0_0_3: u32 = 0x01000003;
}

impl ReadableFile for Terrain {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Terrain> {
        let mut cursor = Cursor::new(buffer);
        Ok(Terrain::read_options(
            &mut cursor,
            platform.endianness(),
            (),
        )?)
    }
}

impl WritableFile for Terrain {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

impl Terrain {
    /// Returns the real position of this plate, taking into account its size.
    pub fn plate_position(&self, plate: &Plate) -> [f32; 2] {
        [
            self.plate_size as f32 * (plate.x as f32 + 0.5),
            self.plate_size as f32 * (plate.y as f32 + 0.5),
        ]
    }

    /// Returns the filename of the `.mdl` for a given plate index.
    pub fn mdl_filename(plate_index: usize) -> String {
        format!("{:04}.mdl", plate_index)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::read, path::PathBuf};

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Terrain>();
    }

    #[test]
    fn test_simple() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("simple.tera");

        let simple_tera = &read(d).unwrap();
        let tera = Terrain {
            version: Terrain::VERSION_V1_0_0_3,
            plate_size: 128,
            clip_distance: 0.0,
            edge_bias: 1.0,
            sampler_bias: 0,
            plates: vec![
                Plate { x: -1, y: -1 },
                Plate { x: 0, y: -1 },
                Plate { x: -1, y: 0 },
                Plate { x: 0, y: 0 },
            ],
        };

        assert_eq!(*simple_tera, tera.write_to_buffer(Platform::Win32).unwrap());
    }
}
