// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use crate::ReadableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[brw(magic = b"fcsv0100")]
#[derive(Debug)]
pub struct Fdt {
    font_table_header_offset: u32,
    #[brw(pad_after = 16)] // empty
    kerning_header_offset: u32,
}

impl ReadableFile for Fdt {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Self::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Fdt>();
    }
}
