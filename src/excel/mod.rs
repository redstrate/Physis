// SPDX-FileCopyrightText: 2026 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

//! Higher-level Excel API.
//!
//! This module contains types used to examine and edit the game's Excel data. This is mainly accomplished through [Sheet], which provides access over pages of row or subrow data.
//!
//! For the underlying filetypes, see the [EXD](crate::exd), [EXH](crate::exh) and [EXL](crate::exl) modules.
//!
//! # Usage
//!
//! Sheets cannot be created yourself, but instead through [ResourceResolver::read_excel_sheet](crate::resource::ResourceResolver::read_excel_sheet)/[SqPackResource::read_excel_sheet](crate::resource::ResourceResolver::read_excel_sheet). But first, you need to use [ResourceResolver::read_excel_sheet_header](crate::resource::ResourceResolver::read_excel_sheet_header)/[SqPackResource::read_excel_sheet_header](crate::resource::SqPackResource::read_excel_sheet_header) to read [the sheet's metadata](crate::exh::EXH). This informs you what languages are supported, whether the sheet has subrows, etc.
//!
//! ```no_run
//! # use physis::resource::{UnpackedResource, ResourceResolver};
//! # use physis::common::Language;
//! # use physis::Error;
//! # let resource = UnpackedResource::from_existing(".");
//! # let mut resolver = ResourceResolver::new();
//! # resolver.add_source(resource);
//! let exh = resolver.read_excel_sheet_header("Item")?;
//! let sheet = resolver.read_excel_sheet(&exh, "Item", Language::English)?;
//! let row = sheet.row(1); // 1 is the ID for Gil
//! # Ok::<(), Error>(())
//! ```
//!
//! With a Sheet in hand, the most important functions are [Sheet::row] and [Sheet::subrow]. These functions return the row associated with that ID.
//!
//! Each column is a [Field] which is a variant of a few data types. The columns are given to you as-is, determining their actual meaning is outside the scope of this library. One solution is [Icarus](https://github.com/redstrate/Icarus), which derives structured sheets from [EXDSchema](https://github.com/xivdev/EXDSchema) and is based on types from this module.
//!
//! # Iterators
//!
//! We have a variety of ways to iterate through sheets. Here's a complex use-case, showcased by iterating through `SwitchTalkVariation` with its various subrows:
//!
//! ```no_run
//! # use physis::excel::Sheet;
//! # use physis::exh::EXH;
//! # let switch_talk_variation_sheet = Sheet { exh: EXH::new(), pages: Vec::new() };
//! for (_, subrows) in &switch_talk_variation_sheet {
//!    // We need to read these variations from highest to lowest (16..15..14..).
//!    for (subrow_id, subrow) in subrows.iter().rev() {
//!        println!("{subrow_id}: {subrow:#?}");
//!    }
//! }
//! ```
//!
//! But most sheets don't have subrows, so having to deal with nesting is annoying. Use [PageIterator::flatten_subrows] to throw away this information:
//!
//! ```no_run
//! # use physis::excel::Sheet;
//! # use physis::exh::EXH;
//! # let item_sheet = Sheet { exh: EXH::new(), pages: Vec::new() };
//! for (row_id, row) in item_sheet.pages[0].into_iter().flatten_subrows() {
//!     println!("{row_id}: {row:#?}");
//! }
//! ```
//!
//! So far our iterators will automatically go to the next [Page] of a [Sheet], but you can also iterate pages by themselves:
//!
//! ```no_run
//! # use physis::excel::Sheet;
//! # use physis::exh::EXH;
//! # let item_sheet = Sheet { exh: EXH::new(), pages: Vec::new() };
//! // We only care about the first page:
//! for (row_id, row) in &item_sheet.pages[0] {
//!     println!("{row_id}: {row:#?}");
//! }
//! ```

use std::io::{Cursor, Seek, SeekFrom};

use binrw::{BinRead, BinWrite};

use crate::{
    ByteBuffer,
    exd::{DataSectionHeader, EXD, EXDHeader, ExcelDataOffset, SubRowHeader},
    exd_file_operations::{read_row, write_row},
    exh::{EXH, SheetRowKind},
};

mod iterators;
pub use iterators::*;

/// Contains a single column's data, which can be various underlying types.
#[derive(Debug, Clone, PartialEq)]
pub enum Field {
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

impl Field {
    /// Returns a `Some(String)` if this column was a `String`, otherwise `None`.
    pub fn into_string(&self) -> Option<&String> {
        if let Field::String(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(bool)` if this column was a `Bool`, otherwise `None`.
    pub fn into_bool(&self) -> Option<&bool> {
        if let Field::Bool(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i8)` if this column was a `Int8`, otherwise `None`.
    pub fn into_i8(&self) -> Option<&i8> {
        if let Field::Int8(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u8)` if this column was a `UInt8`, otherwise `None`.
    pub fn into_u8(&self) -> Option<&u8> {
        if let Field::UInt8(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i16)` if this column was a `Int16`, otherwise `None`.
    pub fn into_i16(&self) -> Option<&i16> {
        if let Field::Int16(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u16)` if this column was a `UInt16`, otherwise `None`.
    pub fn into_u16(&self) -> Option<&u16> {
        if let Field::UInt16(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i32)` if this column was a `Int32`, otherwise `None`.
    pub fn into_i32(&self) -> Option<&i32> {
        if let Field::Int32(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u32)` if this column was a `UInt32`, otherwise `None`.
    pub fn into_u32(&self) -> Option<&u32> {
        if let Field::UInt32(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(f32)` if this column was a `Float32`, otherwise `None`.
    pub fn into_f32(&self) -> Option<&f32> {
        if let Field::Float32(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(i64)` if this column was a `Int64`, otherwise `None`.
    pub fn into_i64(&self) -> Option<&i64> {
        if let Field::Int64(value) = self {
            return Some(value);
        }
        None
    }

    /// Returns a `Some(u64)` if this column was a `UInt64`, otherwise `None`.
    pub fn into_u64(&self) -> Option<&u64> {
        if let Field::UInt64(value) = self {
            return Some(value);
        }
        None
    }
}

/// A single row of data.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// The columns in this row.
    pub columns: Vec<Field>,
}

/// Represents an entry in the page, which is made up of one to multiple subrows.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The row ID.
    pub id: u32,
    /// A list of subrows. The first value of this tuple is the subrow ID.
    pub subrows: Vec<(u16, Row)>,
}

/// A page of rows, inside of a [Sheet].
#[derive(Debug, Clone)]
pub struct Page {
    exd: EXD,

    /// The row descriptors for this page.
    ///
    /// You most likely don't want to use this, prefer [Sheet::row] or [Sheet::subrow] instead.
    // (NOTE: This is currently public because of write support, but this may change in the future.)
    pub entries: Vec<Entry>,
}

impl Page {
    pub(crate) fn from_exd(exh: &EXH, exd: EXD) -> Self {
        let descriptors = Self::read_descriptors(exh, &exd);
        Self {
            exd,
            entries: descriptors,
        }
    }

    fn read_descriptors(exh: &EXH, exd: &EXD) -> Vec<Entry> {
        let mut cursor = Cursor::new(&exd.remaining_data);
        let header_offset = EXDHeader::SIZE as u64 + exd.header.data_offset_size as u64;
        let mut rows = Vec::new();

        for offset in &exd.data_offsets {
            cursor
                .seek(SeekFrom::Start(offset.offset as u64 - header_offset))
                .unwrap();

            let row_header = DataSectionHeader::read(&mut cursor).unwrap();

            let data_offset = cursor.stream_position().unwrap();

            let subrows = if exh.header.row_kind == SheetRowKind::SubRows {
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
                rows
            } else {
                vec![(0, read_row(&mut cursor, exh, data_offset).unwrap())]
            };
            rows.push(Entry {
                id: offset.row_id,
                subrows,
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
                (core::mem::size_of::<ExcelDataOffset>() * self.entries.len()) as i64,
            ))
            .unwrap();

        let mut data_offsets = Vec::with_capacity(self.entries.len());

        for row in &self.entries {
            data_offsets.push(ExcelDataOffset {
                row_id: row.id,
                offset: cursor.stream_position().unwrap() as u32,
            });

            // skip row header for now, because we don't know the size yet!
            let row_header_pos = cursor.stream_position().unwrap();

            cursor.seek(SeekFrom::Current(6)).unwrap(); // u32 + u16

            let old_pos = cursor.stream_position().unwrap();

            // write column data
            match &exh.header.row_kind {
                SheetRowKind::SingleRow => {
                    write_row(&mut cursor, exh, &row.subrows.first().unwrap().1)
                }
                SheetRowKind::SubRows => {
                    for (id, row) in &row.subrows {
                        let subrow_header = SubRowHeader { subrow_id: *id };
                        subrow_header.write_ne(&mut cursor).ok()?;

                        write_row(&mut cursor, exh, row);
                    }
                }
            }

            // write strings at the end of column data
            {
                let mut write_row_strings = |row: &Row| {
                    for column in &row.columns {
                        if let Field::String(val) = column {
                            let bytes = val.as_bytes();
                            bytes.write(&mut cursor).unwrap();

                            // nul terminator
                            0u8.write_le(&mut cursor).unwrap();
                        }
                    }
                };

                for (_, row) in &row.subrows {
                    write_row_strings(row);
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

    /// The number of rows in this sheet. Does *not* take into account subrows.
    pub fn row_count(&self) -> usize {
        self.entries.len()
    }
}

/// Represents an Excel sheet, which contains multiple pages of columns separated by either rows or subrows.
///
/// To read a sheet, use [ResourceResolver::read_excel_sheet](crate::resource::ResourceResolver::read_excel_sheet) or [SqPackResource::read_excel_sheet](crate::resource::SqPackResource::read_excel_sheet).
#[derive(Debug, Clone)]
pub struct Sheet {
    /// The EXH for this sheet.
    // (NOTE: This is currently public because of Icarus, but this may change in the future.)
    pub exh: EXH,
    /// The pages for this sheet.
    pub pages: Vec<Page>,
}

impl Sheet {
    /// Returns the entry matching `row_id` and returns a reference to it, otherwise returns [None].
    ///
    /// This is only useful if you need to discriminate between single row and subrow sheets. In most cases, you want to use [row](Self::row) or [subrow](Self::subrow).
    pub fn entry(&self, row_id: u32) -> Option<&Entry> {
        let page_index = self.exh.get_page(row_id);
        let page = self.pages.get(page_index)?;

        page.entries.iter().find(|row| row.id == row_id)
    }

    /// Finds a row matching `row_id` and returns a reference to it, otherwise returns [None].
    ///
    /// For sheets that have subrows, this will always return subrow 0. It's recommended to use [subrow](Self::subrow) instead.
    pub fn row(&self, row_id: u32) -> Option<&Row> {
        let row = self.entry(row_id)?;
        row.subrows.first().map(|(_, row)| row)
    }

    /// Finds a row matching `row_id` and `subrow_id` and returns a reference to it, otherwise returns [None].
    ///
    /// For sheets that don't have subrows, this will always return [None].
    pub fn subrow(&self, row_id: u32, subrow_id: u16) -> Option<&Row> {
        // Grabbing a subrow here never makes sense, and is just misuse of the API.
        if self.exh.header.row_kind == SheetRowKind::SingleRow {
            return None;
        }

        let row = self.entry(row_id)?;
        row.subrows
            .iter()
            .find(|(id, _)| *id == subrow_id)
            .map(|(_, row)| row)
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

        let page = Page::from_exd(&exh, exd);

        let excel = Sheet {
            exh,
            pages: vec![page],
        };

        assert_eq!(excel.pages[0].entries.len(), 4);

        // row 0
        assert_eq!(
            excel.pages[0].entries[0],
            Entry {
                id: 1441792,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![Field::Int8(0)]
                    }
                )]
            }
        );
        assert!(excel.row(1441792).is_some());

        // row 1
        assert_eq!(
            excel.pages[0].entries[1],
            Entry {
                id: 1441793,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![Field::Int8(1)]
                    }
                )]
            }
        );
        assert!(excel.row(1441793).is_some());

        // row 2
        assert_eq!(
            excel.pages[0].entries[2],
            Entry {
                id: 1441794,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![Field::Int8(2)]
                    }
                )]
            }
        );
        assert!(excel.row(1441794).is_some());

        // row 3
        assert_eq!(
            excel.pages[0].entries[3],
            Entry {
                id: 1441795,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![Field::Int8(3)]
                    }
                )]
            }
        );
        assert!(excel.row(1441795).is_some());

        // non-existent row 4
        assert!(excel.row(1019181719).is_none());
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

        let page = Page::from_exd(&exh, exd);

        let excel = Sheet {
            exh,
            pages: vec![page],
        };

        assert_eq!(excel.pages[0].entries.len(), 8);

        // row 0
        assert_eq!(
            excel.pages[0].entries[0],
            Entry {
                id: 0,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("HOWTO_MOVE_AND_CAMERA".to_string()),
                            Field::UInt32(1)
                        ]
                    }
                )]
            }
        );

        // row 1
        assert_eq!(
            excel.pages[0].entries[1],
            Entry {
                id: 1,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("HOWTO_ANNOUNCE_AND_QUEST".to_string()),
                            Field::UInt32(2)
                        ]
                    }
                )]
            }
        );

        // row 2
        assert_eq!(
            excel.pages[0].entries[2],
            Entry {
                id: 2,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("HOWTO_QUEST_REWARD".to_string()),
                            Field::UInt32(11)
                        ]
                    }
                )]
            }
        );

        // row 3
        assert_eq!(
            excel.pages[0].entries[3],
            Entry {
                id: 3,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("BGM_MUSIC_NO_MUSIC".to_string()),
                            Field::UInt32(1001)
                        ]
                    }
                )]
            }
        );

        // row 4
        assert_eq!(
            excel.pages[0].entries[4],
            Entry {
                id: 4,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("ITEM_INITIAL_RING_A".to_string()),
                            Field::UInt32(4423)
                        ]
                    }
                )]
            }
        );

        // row 5
        assert_eq!(
            excel.pages[0].entries[5],
            Entry {
                id: 5,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("ITEM_INITIAL_RING_B".to_string()),
                            Field::UInt32(4424)
                        ]
                    }
                )]
            }
        );

        // row 6
        assert_eq!(
            excel.pages[0].entries[6],
            Entry {
                id: 6,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("ITEM_INITIAL_RING_C".to_string()),
                            Field::UInt32(4425)
                        ]
                    }
                )]
            }
        );

        // row 7
        assert_eq!(
            excel.pages[0].entries[7],
            Entry {
                id: 7,
                subrows: vec![(
                    0,
                    Row {
                        columns: vec![
                            Field::String("ITEM_INITIAL_RING_D".to_string()),
                            Field::UInt32(4426)
                        ]
                    }
                )]
            }
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

        let page = Page::from_exd(&exh, expected_exd);

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

        let page = Page::from_exd(&exh, expected_exd);

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

        let page = Page::from_exd(&exh, expected_exd);

        let actual_exd_bytes = page.write_to_buffer(&exh).unwrap();
        assert_eq!(actual_exd_bytes, expected_exd_bytes);
    }
}
