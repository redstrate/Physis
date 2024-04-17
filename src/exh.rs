// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use binrw::BinRead;
use binrw::binrw;

use crate::common::Language;
use crate::ByteSpan;

#[binrw]
#[brw(magic = b"EXHF")]
#[brw(big)]
#[allow(dead_code)]
pub struct EXHHeader {
    pub(crate) version: u16,

    pub data_offset: u16,
    pub(crate) column_count: u16,
    pub(crate) page_count: u16,
    pub(crate) language_count: u16,

    #[br(pad_before = 6)]
    #[br(pad_after = 8)]
    pub row_count: u32,
}

#[binrw]
#[brw(repr(u16))]
pub enum ColumnDataType {
    String = 0x0,
    Bool = 0x1,
    Int8 = 0x2,
    UInt8 = 0x3,
    Int16 = 0x4,
    UInt16 = 0x5,
    Int32 = 0x6,
    UInt32 = 0x7,
    Float32 = 0x9,
    Int64 = 0xA,
    UInt64 = 0xB,

    PackedBool0 = 0x19,
    PackedBool1 = 0x1A,
    PackedBool2 = 0x1B,
    PackedBool3 = 0x1C,
    PackedBool4 = 0x1D,
    PackedBool5 = 0x1E,
    PackedBool6 = 0x1F,
    PackedBool7 = 0x20,
}

#[binrw]
#[brw(big)]
pub struct ExcelColumnDefinition {
    pub data_type: ColumnDataType,
    pub offset: u16,
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
pub struct ExcelDataPagination {
    pub start_id: u32,
    pub row_count: u32,
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
pub struct EXH {
    pub header: EXHHeader,

    #[br(count = header.column_count)]
    pub column_definitions: Vec<ExcelColumnDefinition>,

    #[br(count = header.page_count)]
    pub pages: Vec<ExcelDataPagination>,

    #[br(count = header.language_count)]
    pub languages: Vec<Language>,
}

impl EXH {
    pub fn from_existing(buffer: ByteSpan) -> Option<EXH> {
        EXH::read(&mut Cursor::new(&buffer)).ok()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        EXH::from_existing(&read(d).unwrap());
    }
}

