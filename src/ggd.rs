// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use crate::common_file_operations::Half3;
use binrw::BinRead;
use binrw::BinResult;
use binrw::binrw;

/// Grass zone data file, usually with the `.gzd` file extension.
#[binrw]
#[derive(Debug, Clone)]
#[brw(magic = b" dgg")]
pub struct GrassGridData {
    pub version: u32,

    #[br(temp)]
    #[bw(calc = cells.len() as u16)]
    cell_count: u16,

    /// Three per-auto-layer SSAO-mask controls
    pub auto_layer_ssao_control_indices: u16, // TODO: parse

    cell_offsets: [u32; 8],

    // Per-auto-layer random lateral offset ranges
    // The result is multiplied by placement.shape_variation / 255
    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub lateral_offset_min: [f32; 3],
    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub lateral_offset_max: [f32; 3],

    // Per-auto-layer random yaw range
    // Values in radians
    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub yaw_min: [f32; 3],
    #[br(map = |x: Half3| { [x.r.to_f32(), x.g.to_f32(), x.b.to_f32()] })]
    pub yaw_max: [f32; 3],

    // Added to every placement position
    pub world_origin: [f32; 3],

    #[br(if(version >= Self::VERSION_2_0_5_0))]
    pub alignment_bend_weight: [u8; 8],

    #[br(if(version >= Self::VERSION_2_0_5_0))]
    pub alignment_length_gain: [u8; 8],

    #[br(parse_with = parse_cells, args(&cell_offsets, cell_count as usize))]
    pub cells: Vec<GGDCell>,
}

impl GrassGridData {
    pub const VERSION_2_0_5_0: u32 = 0x02000500;
}

impl ReadableFile for GrassGridData {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(Self::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

#[binrw::parser(reader, endian)]
fn parse_cells(offsets: &[u32], cell_count: usize) -> BinResult<Vec<GGDCell>> {
    let mut cells = Vec::new();
    for offset in offsets.iter().take(cell_count) {
        if *offset != 0 {
            reader.seek(SeekFrom::Start(*offset as u64))?;
            cells.push(GGDCell::read_options(reader, endian, ())?);
        }
    }

    Ok(cells)
}

#[binrw]
#[brw(magic = b"dgs\0")]
#[derive(Debug, Clone)]
pub struct GGDCell {
    pub world_bounds_min: [f32; 3],
    pub world_bounds_max: [f32; 3],
    // TODO: parse payloads
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<GrassGridData>();
    }
}
