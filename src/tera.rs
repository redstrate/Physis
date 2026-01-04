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
    pub x: i16,
    pub y: i16,
}

/// Terrain file, usually with the `.tera` file extension.
///
/// Contains a list of plates that make up the underlying terrain of a level.
#[binrw]
#[derive(Debug, Clone)]
pub struct Terrain {
    // Example: 0x1000003
    version: u32,

    #[bw(calc = plates.len() as u32)]
    #[br(temp)]
    plate_count: u32,

    /// Size of each plate in units.
    pub plate_size: u32,
    pub clip_distance: f32,

    unknown: f32,

    #[brw(pad_before = 32)] // empty padding
    #[br(count = plate_count)]
    pub plates: Vec<Plate>,
}

impl ReadableFile for Terrain {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Terrain> {
        let mut cursor = Cursor::new(buffer);
        Terrain::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for Terrain {
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())
                .ok()?;
        }

        Some(buffer)
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
            version: 16777219,
            plate_size: 128,
            clip_distance: 0.0,
            unknown: 1.0,
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
