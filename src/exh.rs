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
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Language;
use crate::common::Platform;

/// What kind of rows this Excel sheet has.
#[binrw]
#[derive(Debug, PartialEq, Eq)]
#[brw(repr = u8)]
pub enum SheetRowKind {
    /// Single rows.
    SingleRow = 1,
    /// Rows with subrows.
    SubRows = 2,
}

/// Header for EXH files.
#[binrw]
#[brw(magic = b"EXHF")]
#[brw(big)]
#[derive(Debug)]
pub struct EXHHeader {
    pub(crate) version: u16,

    pub(crate) row_size: u16,
    pub(crate) column_count: u16,
    pub(crate) page_count: u16,
    pub(crate) language_count: u16,

    /// Usually 0
    pub(crate) unk1: u16,

    pub(crate) unk2: u8,

    /// Whether this Excel sheet uses subrows or just single rows.
    pub row_kind: SheetRowKind,

    pub(crate) unk3: u16,

    /// How many rows are in this Excel sheet.
    #[brw(pad_after = 8)] // padding
    pub row_count: u32,
}

/// Data type for a column.
#[binrw]
#[brw(repr(u16))]
#[repr(u16)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ColumnDataType {
    /// String.
    String = 0x0,
    /// Boolean.
    Bool = 0x1,
    /// 8-bit signed integer.
    Int8 = 0x2,
    /// 8-bit unsigned integer.
    UInt8 = 0x3,
    /// 16-bit signed integer.
    Int16 = 0x4,
    /// 16-bit unsigned integer.
    UInt16 = 0x5,
    /// 32-bit signed integer.
    Int32 = 0x6,
    /// 32-bit unsigned integer.
    UInt32 = 0x7,
    /// 32-bit floating point.
    Float32 = 0x9,
    /// 64-bit signed integer.
    Int64 = 0xA,
    /// 64-bit unsigned integer.
    UInt64 = 0xB,

    /// Packed boolean (0 index).
    PackedBool0 = 0x19,
    /// Packed boolean (1 index).
    PackedBool1 = 0x1A,
    /// Packed boolean (2 index).
    PackedBool2 = 0x1B,
    /// Packed boolean (3 index).
    PackedBool3 = 0x1C,
    /// Packed boolean (4 index).
    PackedBool4 = 0x1D,
    /// Packed boolean (5 index).
    PackedBool5 = 0x1E,
    /// Packed boolean (6 index).
    PackedBool6 = 0x1F,
    /// Packed boolean (7 index).
    PackedBool7 = 0x20,
}

/// A column in an Excel sheet.
#[binrw]
#[brw(big)]
#[derive(Debug, Copy, Clone)]
pub struct ExcelColumnDefinition {
    /// What data type this column is.
    pub data_type: ColumnDataType,
    /// The offset from the row's beginning, in bytes.
    pub offset: u16,
}

/// Page in an Excel sheet.
#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct ExcelDataPagination {
    /// Which ID do rows start at.
    pub start_id: u32,
    /// How many rows are in this page.
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
    /// Header for this file.
    pub header: EXHHeader,

    /// Columns and their types.
    #[br(count = header.column_count)]
    pub column_definitions: Vec<ExcelColumnDefinition>,

    /// Page information.
    #[br(count = header.page_count)]
    pub pages: Vec<ExcelDataPagination>,

    /// Supported languages.
    #[br(count = header.language_count)]
    #[brw(pad_after = 1)] // \0
    pub languages: Vec<Language>,
}

impl ReadableFile for EXH {
    fn from_existing(_platform: Platform, buffer: ByteSpan) -> Option<Self> {
        Self::read(&mut Cursor::new(&buffer)).ok()
    }
}

impl WritableFile for EXH {
    fn write_to_buffer(&self, _platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            self.write_args(&mut writer, ()).unwrap();
        }

        Some(buffer)
    }
}

impl EXH {
    /// Returns the page that contains this `row_id`.
    pub(crate) fn get_page(&self, row_id: u32) -> usize {
        for (i, page) in self.pages.iter().enumerate() {
            if row_id >= page.start_id && row_id < page.start_id + page.row_count {
                return i;
            }
        }

        0
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<EXH>();
    }

    // simple EXH to read, just one page
    #[test]
    fn test_read() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("gcshop.exh");

        let exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();

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
        let expected_exh = EXH::from_existing(Platform::Win32, &expected_exh_bytes).unwrap();

        let actual_exh_bytes = expected_exh.write_to_buffer(Platform::Win32).unwrap();

        assert_eq!(actual_exh_bytes, expected_exh_bytes);
    }
}
