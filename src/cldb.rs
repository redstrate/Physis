// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::binrw;
use binrw::{BinRead, BinWrite};

/// cloud data binary file, usually with the `.cldb` file extension.
#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct CloudData {
    version: u32,
    #[brw(pad_after = 3)] // padding
    positive_param_index: u8,
    #[br(temp)]
    #[bw(calc = spheres.len() as u16)]
    sphere_count: u16,
    sphere_offset: u16,
    unk: [u8; 20],
    unknown20: f32,
    unknown24: f32,
    #[br(count = sphere_offset - 10)]
    unknown_data: Vec<u8>,
    #[br(seek_before = SeekFrom::Start(sphere_offset as u64 * 4), count = sphere_count)]
    pub spheres: Vec<CloudAssetSphere>,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct CloudAssetSphere {
    pub position: [f32; 3],
    pub radius: f32,
    unknown10: f32,
    unknown14: f32,
    unknown18: f32,
    unknown1c: f32,
}

impl ReadableFile for CloudData {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);

        Ok(Self::read_options(&mut cursor, endianness, ())?)
    }
}

impl WritableFile for CloudData {
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
        pass_random_invalid::<CloudData>();
    }
}
