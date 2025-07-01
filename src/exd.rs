// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::{BufWriter, Cursor};

use binrw::BinRead;
use binrw::{BinWrite, binrw};

use crate::common::Language;
use crate::exd_file_operations::{parse_rows, read_data_sections, write_rows};
use crate::exh::{EXH, ExcelDataPagination};
use crate::{ByteBuffer, ByteSpan};

#[binrw]
#[brw(magic = b"EXDF")]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct EXDHeader {
    /// Usually 2, I don't think I've seen any other version
    pub(crate) version: u16,
    /// Seems to be 0?
    pub(crate) unk1: u16,
    /// Size of the data offsets in bytes
    pub(crate) data_offset_size: u32,
    #[brw(pad_after = 16)] // padding
    /// Size of the data sections in bytes
    pub(crate) data_section_size: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
pub(crate) struct ExcelDataOffset {
    /// The row ID associated with this data offset
    pub(crate) row_id: u32,
    /// Offset to it's data section in bytes from the start of the file.
    pub(crate) offset: u32,
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct DataSectionHeader {
    /// Size of the data section in bytes.
    pub(crate) size: u32,
    /// The number of rows in this data section.
    pub(crate) row_count: u16,
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct SubRowHeader {
    pub(crate) subrow_id: u16,
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct DataSection {
    /// Header of the section.
    pub(crate) header: DataSectionHeader,
    /// Data part of this section.
    #[br(temp, count = header.size)]
    #[bw(ignore)]
    data: Vec<u8>,
}

/// An Excel data file. Represents a page in an Excel Sheet.
#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
#[brw(import(exh: &EXH))]
pub struct EXD {
    header: EXDHeader,

    #[br(count = header.data_offset_size / core::mem::size_of::<ExcelDataOffset>() as u32)]
    #[bw(ignore)] // write_rows handles writing this
    data_offsets: Vec<ExcelDataOffset>,

    #[br(parse_with = read_data_sections, args(&header))]
    #[bw(ignore)] // write_rows handles writing this
    data: Vec<DataSection>,

    /// The rows contained in this EXD.
    #[br(parse_with = parse_rows, args(exh, &data_offsets))]
    #[bw(write_with = write_rows, args(exh))]
    pub rows: Vec<ExcelRow>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnData {
    String(String),
    Bool(bool),
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Float32(f32),
    Int64(i64),
    UInt64(u64),
}

impl ColumnData {
    // Returns a Some(String) if this column was a String, otherwise None.
    pub fn into_string(&self) -> Option<&String> {
        if let ColumnData::String(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(bool) if this column was a Bool, otherwise None.
    pub fn into_bool(&self) -> Option<&bool> {
        if let ColumnData::Bool(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(i8) if this column was a Int8, otherwise None.
    pub fn into_i8(&self) -> Option<&i8> {
        if let ColumnData::Int8(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(u8) if this column was a UInt8, otherwise None.
    pub fn into_u8(&self) -> Option<&u8> {
        if let ColumnData::UInt8(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(i16) if this column was a Int16, otherwise None.
    pub fn into_i16(&self) -> Option<&i16> {
        if let ColumnData::Int16(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(u16) if this column was a UInt16, otherwise None.
    pub fn into_u16(&self) -> Option<&u16> {
        if let ColumnData::UInt16(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(i32) if this column was a Int32, otherwise None.
    pub fn into_i32(&self) -> Option<&i32> {
        if let ColumnData::Int32(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(u32) if this column was a UInt32, otherwise None.
    pub fn into_u32(&self) -> Option<&u32> {
        if let ColumnData::UInt32(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(f32) if this column was a Float32, otherwise None.
    pub fn into_f32(&self) -> Option<&f32> {
        if let ColumnData::Float32(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(i64) if this column was a Int64, otherwise None.
    pub fn into_i64(&self) -> Option<&i64> {
        if let ColumnData::Int64(value) = self {
            return Some(value);
        }
        None
    }

    // Returns a Some(u64) if this column was a UInt64, otherwise None.
    pub fn into_u64(&self) -> Option<&u64> {
        if let ColumnData::UInt64(value) = self {
            return Some(value);
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExcelSingleRow {
    pub columns: Vec<ColumnData>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExcelRowKind {
    SingleRow(ExcelSingleRow),
    SubRows(Vec<(u16, ExcelSingleRow)>),
}

/// Represents an entry in the EXD.
#[derive(Debug)]
pub struct ExcelRow {
    /// The row ID associated with this entry.
    pub row_id: u32,
    /// The kind of entry.
    pub kind: ExcelRowKind,
}

impl EXD {
    /// Parse an EXD from an existing file.
    pub fn from_existing(exh: &EXH, buffer: ByteSpan) -> Option<EXD> {
        EXD::read_args(&mut Cursor::new(&buffer), (exh,)).ok()
    }

    /// Finds the entry with the specified ID, otherwise returns `None`.
    pub fn get_row(&self, row_id: u32) -> Option<ExcelRowKind> {
        for row in &self.rows {
            if row.row_id == row_id {
                return Some(row.kind.clone());
            }
        }

        None
    }

    /// Finds the entry with the specified ID, otherwise returns `None`.
    pub fn get_subrow(&self, row_id: u32, subrow_id: u16) -> Option<ExcelSingleRow> {
        for row in &self.rows {
            if row.row_id == row_id {
                match &row.kind {
                    ExcelRowKind::SingleRow(_) => {}
                    ExcelRowKind::SubRows(subrows) => {
                        dbg!(subrows);
                        if let Some(subrow) =
                            subrows.iter().filter(|(id, _)| *id == subrow_id).next()
                        {
                            return Some(subrow.1.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Calculate the filename of an EXD from the `name`, `language`, and `page`.
    pub fn calculate_filename(
        name: &str,
        language: Language,
        page: &ExcelDataPagination,
    ) -> String {
        use crate::common::get_language_code;

        match language {
            Language::None => {
                format!("{name}_{}.exd", page.start_id)
            }
            lang => {
                format!("{name}_{}_{}.exd", page.start_id, get_language_code(&lang))
            }
        }
    }

    /// Write this EXD back to it's serialized binary form.
    pub fn write_to_buffer(&self, exh: &EXH) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            self.write_args(&mut writer, (exh,)).unwrap();
        }

        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::exh::EXHHeader;
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        let exh = EXH {
            header: EXHHeader {
                version: 0,
                row_size: 0,
                column_count: 0,
                page_count: 0,
                language_count: 0,
                row_count: 0,
                unk1: 0,
            },
            column_definitions: vec![],
            pages: vec![],
            languages: vec![],
        };

        // Feeding it invalid data should not panic
        EXD::from_existing(&exh, &read(d).unwrap());
    }

    // super simple EXD to read, it's just a few rows of only int8's
    #[test]
    fn test_read() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop.exh");

            exh = EXH::from_existing(&read(d).unwrap()).unwrap();
        }

        // exd
        let exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop_1441792.exd");

            exd = EXD::from_existing(&exh, &read(d).unwrap()).unwrap();
        }

        assert_eq!(exd.rows.len(), 4);

        // row 0
        assert_eq!(exd.rows[0].row_id, 1441792);
        assert_eq!(
            exd.rows[0].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(0)]
            })
        );

        // row 1
        assert_eq!(exd.rows[1].row_id, 1441793);
        assert_eq!(
            exd.rows[1].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(1)]
            })
        );

        // row 2
        assert_eq!(exd.rows[2].row_id, 1441794);
        assert_eq!(
            exd.rows[2].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(2)]
            })
        );

        // row 3
        assert_eq!(exd.rows[3].row_id, 1441795);
        assert_eq!(
            exd.rows[3].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(3)]
            })
        );
    }

    // super simple EXD to write, it's just a few rows of only int8's
    #[test]
    fn test_write() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop.exh");

            exh = EXH::from_existing(&read(d).unwrap()).unwrap();
        }

        // exd
        let expected_exd_bytes;
        let expected_exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop_1441792.exd");

            expected_exd_bytes = read(d).unwrap();
            expected_exd = EXD::from_existing(&exh, &expected_exd_bytes).unwrap();
        }

        let actual_exd_bytes = expected_exd.write_to_buffer(&exh).unwrap();
        assert_eq!(actual_exd_bytes, expected_exd_bytes);
    }

    // slightly more complex to read, because it has STRINGS
    #[test]
    fn test_read_strings() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("openingsystemdefine.exh");

            exh = EXH::from_existing(&read(d).unwrap()).unwrap();
        }

        // exd
        let exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("openingsystemdefine_0.exd");

            exd = EXD::from_existing(&exh, &read(d).unwrap()).unwrap();
        }

        assert_eq!(exd.rows.len(), 8);

        // row 0
        assert_eq!(exd.rows[0].row_id, 0);
        assert_eq!(
            exd.rows[0].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("HOWTO_MOVE_AND_CAMERA".to_string()),
                    ColumnData::UInt32(1)
                ]
            })
        );

        // row 1
        assert_eq!(exd.rows[1].row_id, 1);
        assert_eq!(
            exd.rows[1].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("HOWTO_ANNOUNCE_AND_QUEST".to_string()),
                    ColumnData::UInt32(2)
                ]
            })
        );

        // row 2
        assert_eq!(exd.rows[2].row_id, 2);
        assert_eq!(
            exd.rows[2].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("HOWTO_QUEST_REWARD".to_string()),
                    ColumnData::UInt32(11)
                ]
            })
        );

        // row 3
        assert_eq!(exd.rows[3].row_id, 3);
        assert_eq!(
            exd.rows[3].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("BGM_MUSIC_NO_MUSIC".to_string()),
                    ColumnData::UInt32(1001)
                ]
            })
        );

        // row 4
        assert_eq!(exd.rows[4].row_id, 4);
        assert_eq!(
            exd.rows[4].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_A".to_string()),
                    ColumnData::UInt32(4423)
                ]
            })
        );

        // row 5
        assert_eq!(exd.rows[5].row_id, 5);
        assert_eq!(
            exd.rows[5].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_B".to_string()),
                    ColumnData::UInt32(4424)
                ]
            })
        );

        // row 6
        assert_eq!(exd.rows[6].row_id, 6);
        assert_eq!(
            exd.rows[6].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_C".to_string()),
                    ColumnData::UInt32(4425)
                ]
            })
        );

        // row 7
        assert_eq!(exd.rows[7].row_id, 7);
        assert_eq!(
            exd.rows[7].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_D".to_string()),
                    ColumnData::UInt32(4426)
                ]
            })
        );
    }

    // slightly more complex to write, because it has STRINGS
    #[test]
    fn test_write_strings() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("openingsystemdefine.exh");

            exh = EXH::from_existing(&read(d).unwrap()).unwrap();
        }

        // exd
        let expected_exd_bytes;
        let expected_exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("openingsystemdefine_0.exd");

            expected_exd_bytes = read(d).unwrap();
            expected_exd = EXD::from_existing(&exh, &expected_exd_bytes).unwrap();
        }

        let actual_exd_bytes = expected_exd.write_to_buffer(&exh).unwrap();
        assert_eq!(actual_exd_bytes, expected_exd_bytes);
    }

    // this doesn't have any strings, but a LOT of columns and some packed booleans!
    #[test]
    fn test_write_many_columns() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("physicsgroup.exh");

            exh = EXH::from_existing(&read(d).unwrap()).unwrap();
        }

        // exd
        let expected_exd_bytes;
        let expected_exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("physicsgroup_1.exd");

            expected_exd_bytes = read(d).unwrap();
            expected_exd = EXD::from_existing(&exh, &expected_exd_bytes).unwrap();
        }

        let actual_exd_bytes = expected_exd.write_to_buffer(&exh).unwrap();
        assert_eq!(actual_exd_bytes, expected_exd_bytes);
    }
}
