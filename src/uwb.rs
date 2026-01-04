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
#[derive(Debug)]
#[brw(magic = b"UWC1")]
pub struct Uwc {
    /// Including this header, in bytes.
    /// Seems to be always be 88 bytes.
    #[br(temp)]
    #[bw(calc = Self::SIZE as u32)]
    size: u32,
    unk: [f32; 20],
}

impl Uwc {
    pub const SIZE: usize = 88;
}

impl ReadableFile for Uwb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Uwb::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for Uwb {
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

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Uwb>();
    }
}
