// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::common_file_operations::read_short_identifier;
use crate::common_file_operations::write_short_identifier;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
struct SkpHeader {
    magic: i32, // TODO: what magic?

    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    pub version: String,
}

#[binrw]
#[derive(Debug)]
pub struct Skp {
    header: SkpHeader,
}

impl ReadableFile for Skp {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Skp::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for Skp {
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
        pass_random_invalid::<Skp>();
    }
}
