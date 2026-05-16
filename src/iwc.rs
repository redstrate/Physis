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
#[derive(Debug)]
struct IwcHeader {
    count: u16,
    part_mask: u16,
}

#[binrw]
#[derive(Debug)]
pub struct Iwc {
    header: IwcHeader,
}

impl ReadableFile for Iwc {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Iwc::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for Iwc {
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
        pass_random_invalid::<Iwc>();
    }
}
