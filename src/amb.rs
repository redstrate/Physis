// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::large_enum_variant)] // for now...

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinReaderExt;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::VecArgs;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
pub enum AmbSubtype {
    #[brw(magic = 0u8)]
    AmbientWeather(AmbAmbientWeather),
    #[brw(magic = 1u8)]
    SkylightSkySetDatabase(AmbSkylightSkySetDatabase),
}

/// Ambient set file, usually with the `.amb` file extension.
#[binrw]
#[derive(Debug)]
#[brw(magic = b"AMB\0")]
pub struct Amb {
    /// Should be 1?
    pub version: u16,
    /// Should be 0?
    pub endianness: u8,
    pub data: AmbSubtype,
}

#[binrw]
#[derive(Debug)]
pub struct AmbAmbientWeather {
    pub sky_visibility_spherical_harmonics: SecondOrderSphericalHarmonics,

    inner_counts: [i32; Self::ENTRY_COUNT],

    #[br(parse_with = read_amb_entry, args(&inner_counts,))]
    pub entries: [Vec<AmbWeatherKeyframe>; Self::ENTRY_COUNT],
}

impl AmbAmbientWeather {
    pub(crate) const ENTRY_COUNT: usize = 0x20;
}

#[binrw]
#[derive(Debug)]
pub struct AmbSkylightSkySetDatabase {
    reserved_08: u16,

    #[br(temp)]
    #[bw(calc = sky_sets.len() as u16)]
    sky_set_count: u16,
    #[br(temp)]
    #[bw(calc = samples.len() as u32)]
    total_sample_count: u32,

    #[br(count = sky_set_count)]
    pub sky_sets: Vec<SkylightSkySetDescriptor>,
    #[br(count = total_sample_count)]
    pub samples: Vec<SkylightSphericalHarmonicsSample>,
}

#[binrw]
#[derive(Debug)]
pub struct SecondOrderSphericalHarmonics {
    pub l0_constant: f32,
    pub l1_y: f32,
    pub l1_z: f32,
    pub l1_x: f32,
    pub l2_xy: f32,
    pub l2_yz: f32,
    pub l2_3z2_minus_1: f32,
    pub l2_xz: f32,
    pub l2_x2_minus_y2: f32,
}

#[binrw]
#[derive(Debug)]
pub struct AmbWeatherKeyframe {
    /// Ambient light spherical harmonics.
    pub ambient_light: AmbientLightSphericalHarmonics,
    /// In seconds.
    pub time_of_day: f32,
}

#[binrw]
#[derive(Debug)]
pub struct AmbientLightSphericalHarmonics {
    pub red: SecondOrderSphericalHarmonics,
    pub green: SecondOrderSphericalHarmonics,
    pub blue: SecondOrderSphericalHarmonics,
}

#[binrw::parser(reader, endian)]
fn read_amb_entry(
    counts: &[i32],
) -> BinResult<[Vec<AmbWeatherKeyframe>; AmbAmbientWeather::ENTRY_COUNT]> {
    let mut entries: [Vec<AmbWeatherKeyframe>; AmbAmbientWeather::ENTRY_COUNT] = Default::default();

    for (i, count) in counts.iter().enumerate() {
        let entry: Vec<AmbWeatherKeyframe> =
            reader.read_type_args(endian, VecArgs::builder().count(*count as usize).finalize())?;
        entries[i] = entry;
    }

    Ok(entries)
}

#[binrw]
#[derive(Debug)]
pub struct SkylightSkySetDescriptor {
    pub sky_set_id: u16,
    pub sample_count: u16,
    pub first_sample_index: u32,
}

#[binrw]
#[derive(Debug)]
pub struct SkylightSphericalHarmonicsSample {
    pub ambient_light: AmbientLightSphericalHarmonics,
}

impl ReadableFile for Amb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);

        Ok(Self::read_options(&mut cursor, endianness, ())?)
    }
}

impl WritableFile for Amb {
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
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Amb>();
    }
}
