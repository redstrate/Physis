// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::unnecessary_fallible_conversions)] // This wrongly trips on binrw code
#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::BufWriter;
use std::io::Cursor;

use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::common::Language;

#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(repr = u8)]
pub enum SheetRowKind {
    SingleRow = 1,
    SubRows = 2,
}

#[binrw]
#[brw(magic = b"EXHF")]
#[brw(big)]
#[derive(Debug)]
pub struct EXHHeader {
    pub(crate) version: u16,

    pub row_size: u16,
    pub(crate) column_count: u16,
    pub(crate) page_count: u16,
    pub(crate) language_count: u16,

    /// Usually 0
    pub unk1: u16,

    pub unk2: u8,

    /// Whether this Excel sheet uses subrows or just single rows.
    pub row_kind: SheetRowKind,

    pub unk3: u16,

    #[brw(pad_after = 8)] // padding
    pub row_count: u32,
}

#[binrw]
#[brw(repr(u16))]
#[repr(u16)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
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
#[derive(Debug, Copy, Clone)]
pub struct ExcelColumnDefinition {
    pub data_type: ColumnDataType,
    pub offset: u16,
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct ExcelDataPagination {
    pub start_id: u32,
    pub row_count: u32,
}

/// Excel header file, usually with the `.exh` file extension.
///
/// Contains general information about the sheet, such as which languages are supported.
#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct EXH {
    pub header: EXHHeader,

    #[br(count = header.column_count)]
    pub column_definitions: Vec<ExcelColumnDefinition>,

    #[br(count = header.page_count)]
    pub pages: Vec<ExcelDataPagination>,

    #[br(count = header.language_count)]
    #[brw(pad_after = 1)] // \0
    pub languages: Vec<Language>,
}

impl EXH {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<EXH> {
        EXH::read(&mut Cursor::new(&buffer)).ok()
    }

    /// Writes data back to a buffer.
    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            self.write_args(&mut writer, ()).unwrap();
        }

        Some(buffer)
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

    // simple EXH to read, just one page
    #[test]
    fn test_read() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("gcshop.exh");

        let exh = EXH::from_existing(&read(d).unwrap()).unwrap();

        // header
        assert_eq!(exh.header.version, 3);
        assert_eq!(exh.header.row_size, 4);
        assert_eq!(exh.header.column_count, 1);
        assert_eq!(exh.header.page_count, 1);
        assert_eq!(exh.header.language_count, 1);
        assert_eq!(exh.header.row_count, 4);

        // column definitions
        assert_eq!(exh.column_definitions.len(), 1);
        assert_eq!(exh.column_definitions[0].data_type, ColumnDataType::Int8);
        assert_eq!(exh.column_definitions[0].offset, 0);

        // pages
        assert_eq!(exh.pages.len(), 1);
        assert_eq!(exh.pages[0].start_id, 1441792);
        assert_eq!(exh.pages[0].row_count, 4);

        // languages
        assert_eq!(exh.languages.len(), 1);
        assert_eq!(exh.languages[0], Language::None);
    }

    // simple EXH to write, only one page
    #[test]
    fn test_write() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("gcshop.exh");

        let expected_exh_bytes = read(d).unwrap();
        let expected_exh = EXH::from_existing(&expected_exh_bytes).unwrap();

        let actual_exh_bytes = expected_exh.write_to_buffer().unwrap();

        assert_eq!(actual_exh_bytes, expected_exh_bytes);
    }
}
