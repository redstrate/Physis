// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::common_file_operations::read_string;
use binrw::BinRead;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::binrw;

/// Grass zone data file, usually with the `.gzd` file extension.
#[binrw]
#[derive(Debug, Clone)]
#[brw(magic = b"dzg\0")]
pub struct GrassZoneData {
    pub version: u32,

    #[br(temp)]
    #[bw(calc = high_zones.len() as u16)]
    high_zone_count: u16,
    #[br(temp)]
    #[bw(calc = medium_zones.len() as u16)]
    medium_zone_count: u16,
    #[br(temp)]
    #[bw(calc = low_zones.len() as u16)]
    low_zone_count: u16,

    // This isn't actually used, seems to always be 32
    pub model_slot_capacity: u8,

    #[br(temp)]
    #[bw(calc = model_paths.len() as u8)]
    model_path_count: u8,

    /// For each non-empty string, it loads <gzd-dir>/<basename>.tex
    /// Renderer binds slot N with GGD auto-grass layer N
    /// color_map_u_offset selects within this tex
    #[br(parse_with = parse_auto_layer_color_map)]
    #[bw(ignore)]
    pub auto_layer_color_map: [String; 3],

    #[br(if(version >= Self::VERSION_2_0_6_0))]
    pub auto_layer_values: [f32; 3],

    #[br(parse_with = parse_model_paths, args(model_path_count as usize))]
    #[bw(ignore)]
    pub model_paths: Vec<String>,
    #[br(count = high_zone_count)]
    pub high_zones: Vec<GrassZonePlacement>,
    #[br(count = medium_zone_count)]
    pub medium_zones: Vec<GrassZonePlacement>,
    #[br(count = low_zone_count)]
    pub low_zones: Vec<GrassZonePlacement>,
}

impl GrassZoneData {
    pub const VERSION_2_0_5_0: u32 = 0x02000500;
    pub const VERSION_2_0_6_0: u32 = 0x02000600;

    const COLOR_MAP_TEXTURE_BASENAME_SIZE: usize = 0x20;
    const MODEL_PATH_SIZE: usize = 0x100;
}

impl ReadableFile for GrassZoneData {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(Self::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

impl WritableFile for GrassZoneData {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

#[binrw::parser(reader)]
fn parse_auto_layer_color_map() -> BinResult<[String; 3]> {
    let mut data0: Vec<u8> = vec![0; GrassZoneData::COLOR_MAP_TEXTURE_BASENAME_SIZE];
    reader.read_exact(&mut data0)?;

    let mut data1: Vec<u8> = vec![0; GrassZoneData::COLOR_MAP_TEXTURE_BASENAME_SIZE];
    reader.read_exact(&mut data1)?;

    let mut data2: Vec<u8> = vec![0; GrassZoneData::COLOR_MAP_TEXTURE_BASENAME_SIZE];
    reader.read_exact(&mut data2)?;

    Ok([read_string(data0), read_string(data1), read_string(data2)])
}

#[binrw::parser(reader)]
fn parse_model_paths(model_path_count: usize) -> BinResult<Vec<String>> {
    let mut paths = Vec::new();
    for _ in 0..model_path_count {
        let mut data: Vec<u8> = vec![0; GrassZoneData::MODEL_PATH_SIZE];
        reader.read_exact(&mut data)?;

        paths.push(read_string(data));
    }

    Ok(paths)
}

#[binrw]
#[derive(Debug, Clone)]
pub struct GrassZonePlacement {
    pub center: [f32; 3],
    /// This + center forms a sphere
    pub bounding_sphere_radius: f32,
    pub grid_key: GrassGridKey,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct GrassGridKey {
    pub tier: GrassGridTier,
    pub grid_z: u8,
    pub grid_y: u8,
    pub grid_x: u8,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, Clone)]
pub enum GrassGridTier {
    High = 0,
    Medium = 1,
    Low = 2,
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<GrassZoneData>();
    }
}
