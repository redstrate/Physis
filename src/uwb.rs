// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
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

/// Underwater binary file, usually with the `.uwb` file extension.
#[binrw]
#[derive(Debug)]
#[brw(magic = b"UWB1")]
pub struct Uwb {
    /// Including this header, in bytes.
    #[br(temp)]
    #[bw(calc = self.calculate_file_size())]
    file_size: u32,
    /// Number of UWC's
    #[br(temp)]
    #[bw(calc = uwcs.len() as u32)]
    uwc_count: u32,
    #[br(count = uwc_count)]
    pub uwcs: Vec<Uwc>,
}

impl Uwb {
    fn calculate_file_size(&self) -> u32 {
        12 // UWB1 header
        + (Uwc::SIZE * self.uwcs.len()) as u32
    }
}

#[binrw]
#[derive(Debug, Default)]
#[brw(magic = b"UWC1")]
pub struct Uwc {
    /// Including this header, in bytes.
    /// Seems to be always be 88 bytes.
    #[br(temp)]
    #[bw(calc = Self::SIZE as u32)]
    size: u32,
    pub version: i32,
    /// Height the water surface sits at, which the depth of a point is measured down from.
    pub water_surface_y: f32,
    /// Where the blend from [`fog_shallow`](Self::fog_shallow) to [`fog_deep`](Self::fog_deep)
    /// starts, and how far it runs.
    pub depth_transition_start: f32,
    pub depth_transition_range: f32,
    /// Fog just under the surface.
    pub fog_shallow: UnderwaterFogAttenuation,
    /// Fog past the depth transition.
    pub fog_deep: UnderwaterFogAttenuation,
    /// Where caustics start fading with distance, and how far that fade runs.
    pub caustics_distance_fade_start: f32,
    pub caustics_distance_fade_range: f32,
    /// The two scales the caustic pattern is sampled at.
    pub caustics_uv_size: [f32; 2],
    pub caustics_scroll_speed: f32,
    pub caustics_intensity: f32,
    pub sun_size: f32,
    pub sun_fade_start: f32,
    /// Scales everything lit underwater.
    pub lighting_multiplier: f32,
    /// The client tests this for zero and does nothing else with it.
    pub unknown: u32,
}

#[binrw]
#[derive(Debug, Default)]
pub struct UnderwaterFogAttenuation {
    pub vertical_fade_upper: f32,
    pub vertical_fade_lower: f32,
    pub vertical_attenuation_strength: f32,
}

impl Uwc {
    pub(crate) const SIZE: usize = 88;
}

impl ReadableFile for Uwb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(Uwb::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

impl WritableFile for Uwb {
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
    use crate::{common::ensure_size, pass_random_invalid};

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Uwb>();
    }

    #[test]
    fn test_uwc_size() {
        ensure_size::<Uwc, { Uwc::SIZE }>();
    }
}
