// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use binrw::{BinRead, BinWrite};

use crate::{
    ByteBuffer,
    exd::{DataSectionHeader, EXD, EXDHeader, ExcelDataOffset, SubRowHeader},
    exd_file_operations::{read_row, write_row},
    exh::{EXH, SheetRowKind},
};

/// Contains a single column's data, which can be various underlying types.
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnData {
    /// String.
    String(String),
    /// Boolean.
    Bool(bool),
    /// 8-bit signed integer.
    Int8(i8),
    /// 8-bit unsigned integer.
    UInt8(u8),
    /// 16-bit signed integer.
    Int16(i16),
    /// 16-bit unsigned integer.
    UInt16(u16),
    /// 32-bit signed integer.
    Int32(i32),
    /// 32-bit unsigned integer.
    UInt32(u32),
    /// 32-bit floating point.
    Float32(f32),
    /// 64-bit signed integer.
    Int64(i64),
    /// 64-bit unsigned integer.
    UInt64(u64),
}

impl ColumnData {
    /// Returns a `Some(String)` if this column was a `String`, otherwise `None`.
    pub fn into_string(&self) -> Option<&String> {
        if let ColumnData::String(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(bool)` if this column was a `Bool`, otherwise `None`.
    pub fn into_bool(&self) -> Option<&bool> {
        if let ColumnData::Bool(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i8)` if this column was a `Int8`, otherwise `None`.
    pub fn into_i8(&self) -> Option<&i8> {
        if let ColumnData::Int8(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u8)` if this column was a `UInt8`, otherwise `None`.
    pub fn into_u8(&self) -> Option<&u8> {
        if let ColumnData::UInt8(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i16)` if this column was a `Int16`, otherwise `None`.
    pub fn into_i16(&self) -> Option<&i16> {
        if let ColumnData::Int16(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u16)` if this column was a `UInt16`, otherwise `None`.
    pub fn into_u16(&self) -> Option<&u16> {
        if let ColumnData::UInt16(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i32)` if this column was a `Int32`, otherwise `None`.
    pub fn into_i32(&self) -> Option<&i32> {
        if let ColumnData::Int32(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u32)` if this column was a `UInt32`, otherwise `None`.
    pub fn into_u32(&self) -> Option<&u32> {
        if let ColumnData::UInt32(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(f32)` if this column was a `Float32`, otherwise `None`.
    pub fn into_f32(&self) -> Option<&f32> {
        if let ColumnData::Float32(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i64)` if this column was a `Int64`, otherwise `None`.
    pub fn into_i64(&self) -> Option<&i64> {
        if let ColumnData::Int64(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u64)` if this column was a `UInt64`, otherwise `None`.
    pub fn into_u64(&self) -> Option<&u64> {
        if let ColumnData::UInt64(value) = self {
            return Some(value);
        }
        None
    }
}

/// A single row of Excel data.
// TODO: Rename to ExcelRow
#[derive(Debug, Clone, PartialEq)]
pub struct ExcelSingleRow {
    /// The columns in this row.
    pub columns: Vec<ColumnData>,
}

/// Contains either a single row, or multiple subrows.
#[derive(Debug, Clone, PartialEq)]
pub enum ExcelRowKind {
    /// A single row.
    SingleRow(ExcelSingleRow),
    /// Multiple subrows, with their IDs as the key.
    SubRows(Vec<(u16, ExcelSingleRow)>),
}

/// Represents an entry in the EXD.
#[derive(Debug, Clone)]
pub struct ExcelRow {
    /// Row ID associated with this entry.
    pub row_id: u32,
    /// What kind of entry this represents.
    pub kind: ExcelRowKind,
}

#[derive(Debug, Clone)]
pub struct ExcelSheetPage {
    pub(crate) row_count: u32,
    exd: EXD,

    /// The rows in this page.
    pub rows: Vec<ExcelRow>,
}

impl ExcelSheetPage {
    pub(crate) fn from_exd(page_index: u16, exh: &EXH, exd: EXD) -> Self {
        let rows = Self::get_rows(exh, &exd);
        Self {
            row_count: exh.pages[page_index as usize].row_count,
            exd,
            rows,
        }
    }

    fn get_rows(exh: &EXH, exd: &EXD) -> Vec<ExcelRow> {
        let mut cursor = Cursor::new(&exd.remaining_data);
        let header_offset = EXDHeader::SIZE as u64 + exd.header.data_offset_size as u64;
        let mut rows = Vec::new();

        for offset in &exd.data_offsets {
            cursor
                .seek(SeekFrom::Start(offset.offset as u64 - header_offset))
                .unwrap();

            let row_header = DataSectionHeader::read(&mut cursor).unwrap();

            let data_offset = cursor.stream_position().unwrap();

            let kind = if exh.header.row_kind == SheetRowKind::SubRows {
                let mut rows = Vec::with_capacity(row_header.row_count as usize);
                for i in 0..row_header.row_count {
                    let subrow_offset = data_offset + i as u64 * (2 + exh.header.row_size as u64);
                    cursor.seek(SeekFrom::Start(subrow_offset)).unwrap();

                    let subrow_header = SubRowHeader::read(&mut cursor).unwrap();
                    rows.push((
                        subrow_header.subrow_id,
                        read_row(&mut cursor, exh, subrow_offset + 2).unwrap(),
                    ));
                }
                ExcelRowKind::SubRows(rows)
            } else {
                ExcelRowKind::SingleRow(read_row(&mut cursor, exh, data_offset).unwrap())
            };
            rows.push(ExcelRow {
                row_id: offset.row_id,
                kind,
            });
        }

        rows
    }

    /// Writes EXD data back to a buffer.
    pub fn write_to_buffer(&self, exh: &EXH) -> Option<ByteBuffer> {
        let mut cursor = Cursor::new(Vec::new());

        // Write header
        self.exd.header.write(&mut cursor).ok()?;

        // seek past the data offsets, which we will write later
        let data_offsets_pos = cursor.stream_position().unwrap();
        cursor
            .seek(SeekFrom::Current(
                (core::mem::size_of::<ExcelDataOffset>() * self.rows.len()) as i64,
            ))
            .unwrap();

        let mut data_offsets = Vec::with_capacity(self.rows.len());

        for row in &self.rows {
            data_offsets.push(ExcelDataOffset {
                row_id: row.row_id,
                offset: cursor.stream_position().unwrap() as u32,
            });

            // skip row header for now, because we don't know the size yet!
            let row_header_pos = cursor.stream_position().unwrap();

            cursor.seek(SeekFrom::Current(6)).unwrap(); // u32 + u16

            let old_pos = cursor.stream_position().unwrap();

            // write column data
            match &row.kind {
                ExcelRowKind::SingleRow(excel_single_row) => {
                    write_row(&mut cursor, exh, excel_single_row)
                }
                ExcelRowKind::SubRows(excel_single_rows) => {
                    for (id, row) in excel_single_rows {
                        let subrow_header = SubRowHeader { subrow_id: *id };
                        subrow_header.write_ne(&mut cursor).ok()?;

                        write_row(&mut cursor, exh, row);
                    }
                }
            }

            // write strings at the end of column data
            {
                let mut write_row_strings = |row: &ExcelSingleRow| {
                    for column in &row.columns {
                        if let ColumnData::String(val) = column {
                            let bytes = val.as_bytes();
                            bytes.write(&mut cursor).unwrap();

                            // nul terminator
                            0u8.write_le(&mut cursor).unwrap();
                        }
                    }
                };

                match &row.kind {
                    ExcelRowKind::SingleRow(excel_single_row) => {
                        write_row_strings(excel_single_row)
                    }
                    ExcelRowKind::SubRows(excel_single_rows) => {
                        for (_, row) in excel_single_rows {
                            write_row_strings(row);
                        }
                    }
                }
            }

            // aligned to the next 4 byte boundary
            let boundary_pos = cursor.stream_position().unwrap();
            let remainder = boundary_pos.div_ceil(4) * 4;
            for _ in 0..remainder - boundary_pos {
                0u8.write_le(&mut cursor).unwrap();
            }

            let new_pos = cursor.stream_position().unwrap();

            // write row header
            cursor.seek(SeekFrom::Start(row_header_pos)).unwrap();

            let row_header = DataSectionHeader {
                size: (new_pos - old_pos) as u32,
                row_count: 1, // TODO: hardcoded
            };
            row_header.write(&mut cursor).unwrap();

            // restore pos
            cursor.seek(SeekFrom::Start(new_pos)).unwrap();
        }

        // now write the data offsets
        cursor.seek(SeekFrom::Start(data_offsets_pos)).unwrap();
        data_offsets.write(&mut cursor).unwrap();

        Some(cursor.into_inner())
    }
}

#[derive(Debug, Clone)]
pub struct ExcelSheet {
    /// The EXH for this sheet.
    pub exh: EXH,
    /// All of the pages for this Excel sheet.
    pub pages: Vec<ExcelSheetPage>,
}

impl ExcelSheet {
    /// Finds the entry with the specified `row_id` and returns a reference to it, otherwise returns `None`.
    pub fn get_row(&self, row_id: u32) -> Option<&ExcelRowKind> {
        let page_index = self.exh.get_page(row_id);
        let page = self.pages.get(page_index)?;

        page.rows
            .iter()
            .find(|row| row.row_id == row_id)
            .map(|row| &row.kind)
    }

    /// Finds the entry with the specified `row_id` and `subrow_id` and returns a reference to it, otherwise returns `None`.
    pub fn get_subrow(&self, row_id: u32, subrow_id: u16) -> Option<&ExcelSingleRow> {
        let page_index = self.exh.get_page(row_id);
        let page = self.pages.get(page_index)?;

        let row = page.rows.iter().find(|row| row.row_id == row_id)?;

        match &row.kind {
            ExcelRowKind::SubRows(subrows) => subrows
                .iter()
                .find(|(id, _)| *id == subrow_id)
                .map(|(_, single_row)| single_row),
            ExcelRowKind::SingleRow(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ReadableFile;
    use crate::common::Platform;
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    // super simple EXD to read, it's just a few rows of only int8's
    #[test]
    fn test_read() {
        // exh
        let exh;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop.exh");

            exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        // exd
        let exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop_1441792.exd");

            exd = EXD::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        let page = ExcelSheetPage::from_exd(0, &exh, exd);

        let excel = ExcelSheet {
            exh,
            pages: vec![page],
        };

        assert_eq!(excel.pages[0].rows.len(), 4);

        // row 0
        assert_eq!(excel.pages[0].rows[0].row_id, 1441792);
        assert_eq!(
            excel.pages[0].rows[0].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(0)]
            })
        );
        assert!(excel.get_row(1441792).is_some());

        // row 1
        assert_eq!(excel.pages[0].rows[1].row_id, 1441793);
        assert_eq!(
            excel.pages[0].rows[1].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(1)]
            })
        );
        assert!(excel.get_row(1441793).is_some());

        // row 2
        assert_eq!(excel.pages[0].rows[2].row_id, 1441794);
        assert_eq!(
            excel.pages[0].rows[2].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(2)]
            })
        );
        assert!(excel.get_row(1441794).is_some());

        // row 3
        assert_eq!(excel.pages[0].rows[3].row_id, 1441795);
        assert_eq!(
            excel.pages[0].rows[3].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![ColumnData::Int8(3)]
            })
        );
        assert!(excel.get_row(1441795).is_some());

        // non-existent row 4
        assert!(excel.get_row(1019181719).is_none());
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

            exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        // exd
        let exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("openingsystemdefine_0.exd");

            exd = EXD::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        let page = ExcelSheetPage::from_exd(0, &exh, exd);

        let excel = ExcelSheet {
            exh,
            pages: vec![page],
        };

        assert_eq!(excel.pages[0].rows.len(), 8);

        // row 0
        assert_eq!(excel.pages[0].rows[0].row_id, 0);
        assert_eq!(
            excel.pages[0].rows[0].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("HOWTO_MOVE_AND_CAMERA".to_string()),
                    ColumnData::UInt32(1)
                ]
            })
        );

        // row 1
        assert_eq!(excel.pages[0].rows[1].row_id, 1);
        assert_eq!(
            excel.pages[0].rows[1].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("HOWTO_ANNOUNCE_AND_QUEST".to_string()),
                    ColumnData::UInt32(2)
                ]
            })
        );

        // row 2
        assert_eq!(excel.pages[0].rows[2].row_id, 2);
        assert_eq!(
            excel.pages[0].rows[2].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("HOWTO_QUEST_REWARD".to_string()),
                    ColumnData::UInt32(11)
                ]
            })
        );

        // row 3
        assert_eq!(excel.pages[0].rows[3].row_id, 3);
        assert_eq!(
            excel.pages[0].rows[3].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("BGM_MUSIC_NO_MUSIC".to_string()),
                    ColumnData::UInt32(1001)
                ]
            })
        );

        // row 4
        assert_eq!(excel.pages[0].rows[4].row_id, 4);
        assert_eq!(
            excel.pages[0].rows[4].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_A".to_string()),
                    ColumnData::UInt32(4423)
                ]
            })
        );

        // row 5
        assert_eq!(excel.pages[0].rows[5].row_id, 5);
        assert_eq!(
            excel.pages[0].rows[5].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_B".to_string()),
                    ColumnData::UInt32(4424)
                ]
            })
        );

        // row 6
        assert_eq!(excel.pages[0].rows[6].row_id, 6);
        assert_eq!(
            excel.pages[0].rows[6].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_C".to_string()),
                    ColumnData::UInt32(4425)
                ]
            })
        );

        // row 7
        assert_eq!(excel.pages[0].rows[7].row_id, 7);
        assert_eq!(
            excel.pages[0].rows[7].kind,
            ExcelRowKind::SingleRow(ExcelSingleRow {
                columns: vec![
                    ColumnData::String("ITEM_INITIAL_RING_D".to_string()),
                    ColumnData::UInt32(4426)
                ]
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

            exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        // exd
        let expected_exd_bytes;
        let expected_exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("gcshop_1441792.exd");

            expected_exd_bytes = read(d).unwrap();
            expected_exd = EXD::from_existing(Platform::Win32, &expected_exd_bytes).unwrap();
        }

        let page = ExcelSheetPage::from_exd(0, &exh, expected_exd);

        let actual_exd_bytes = page.write_to_buffer(&exh).unwrap();
        assert_eq!(actual_exd_bytes, expected_exd_bytes);
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

            exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        // exd
        let expected_exd_bytes;
        let expected_exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("openingsystemdefine_0.exd");

            expected_exd_bytes = read(d).unwrap();
            expected_exd = EXD::from_existing(Platform::Win32, &expected_exd_bytes).unwrap();
        }

        let page = ExcelSheetPage::from_exd(0, &exh, expected_exd);

        let actual_exd_bytes = page.write_to_buffer(&exh).unwrap();
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

            exh = EXH::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        }

        // exd
        let expected_exd_bytes;
        let expected_exd;
        {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("physicsgroup_1.exd");

            expected_exd_bytes = read(d).unwrap();
            expected_exd = EXD::from_existing(Platform::Win32, &expected_exd_bytes).unwrap();
        }

        let page = ExcelSheetPage::from_exd(0, &exh, expected_exd);

        let actual_exd_bytes = page.write_to_buffer(&exh).unwrap();
        assert_eq!(actual_exd_bytes, expected_exd_bytes);
    }
}
