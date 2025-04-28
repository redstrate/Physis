// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufWriter, Cursor, Read, Seek, SeekFrom, Write};

use binrw::{binrw, BinResult, BinWrite};
use binrw::{BinRead, Endian};

use crate::{ByteBuffer, ByteSpan};
use crate::common::Language;
use crate::exh::{ColumnDataType, EXH, ExcelColumnDefinition, ExcelDataPagination};

#[binrw]
#[brw(magic = b"EXDF")]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
struct EXDHeader {
    version: u16,

    #[brw(pad_before = 2)] // empty
    /// Size of the ExcelDataOffset array
    index_size: u32,
    #[brw(pad_after = 16)] // empty
    /// Total size of the string data?
    data_section_size: u32,
}

#[binrw]
#[brw(big)]
#[derive(Debug)]
struct ExcelDataOffset {
    row_id: u32,
    pub offset: u32,
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
struct ExcelDataRowHeader {
    data_size: u32,
    row_count: u16,
}

#[binrw::parser(reader)]
fn parse_rows(exh: &EXH, data_offsets: &Vec<ExcelDataOffset>) -> BinResult<Vec<ExcelRow>> {
    let mut rows = Vec::new();

    for offset in data_offsets {
        reader.seek(SeekFrom::Start(offset.offset.into()))?;

        let row_header = ExcelDataRowHeader::read(reader)?;

        let data_offset = reader.stream_position().unwrap() as u32;

        let mut read_row = |row_offset: u32| -> Option<ExcelSingleRow> {
            let mut subrow = ExcelSingleRow {
                columns: Vec::with_capacity(exh.column_definitions.len()),
            };

            for column in &exh.column_definitions {
                reader
                    .seek(SeekFrom::Start((row_offset + column.offset as u32).into()))
                    .ok()?;

                subrow
                    .columns
                    .push(EXD::read_column(reader, exh, column).unwrap());
            }

            Some(subrow)
        };

        let new_row = if row_header.row_count > 1 {
            let mut rows = Vec::new();
            for i in 0..row_header.row_count {
                let subrow_offset =
                    data_offset + (i * exh.header.data_offset + 2 * (i + 1)) as u32;

                rows.push(read_row(subrow_offset).unwrap());
            }
            ExcelRowKind::SubRows(rows)
        } else {
            ExcelRowKind::SingleRow(read_row(data_offset).unwrap())
        };
        rows.push(ExcelRow {
            row_id: offset.row_id,
            kind: new_row
        });
    }

    Ok(rows)
}

#[binrw::writer(writer)]
fn write_rows(
    rows: &Vec<ExcelRow>,
    exh: &EXH,
) -> BinResult<()> {
    // seek past the data offsets, which we will write later
    let data_offsets_pos = writer.stream_position().unwrap();
    writer.seek(SeekFrom::Current((core::mem::size_of::<ExcelDataOffset>() * rows.len()) as i64)).unwrap();

    let mut data_offsets = Vec::new();

    for row in rows {
        data_offsets.push(ExcelDataOffset {
            row_id: row.row_id,
            offset: writer.stream_position().unwrap() as u32
        });

        let row_header = ExcelDataRowHeader {
            data_size: 0,
            row_count: 0
        };
        row_header.write(writer).unwrap();

        // write column data
        {
            let mut write_row = |row: &ExcelSingleRow| {
                for (i, column) in row.columns.iter().enumerate() {
                    EXD::write_column(writer, &column, &exh.column_definitions[i]);
                }
            };

            match &row.kind {
                ExcelRowKind::SingleRow(excel_single_row) => write_row(excel_single_row),
                ExcelRowKind::SubRows(excel_single_rows) => {
                    for row in excel_single_rows {
                        write_row(row);
                    }
                },
            }
        }

        // write strings at the end of column data
        {
            let mut write_row_strings = |row: &ExcelSingleRow| {
                for column in &row.columns {
                    match column {
                        ColumnData::String(val) => {
                            let bytes = val.as_bytes();
                            bytes.write(writer).unwrap();
                        },
                        _ => {}
                    }
                }
            };

            match &row.kind {
                ExcelRowKind::SingleRow(excel_single_row) => write_row_strings(excel_single_row),
                ExcelRowKind::SubRows(excel_single_rows) => {
                    for row in excel_single_rows {
                        write_row_strings(row);
                    }
                },
            }
        }

        // There's an empty byte between each row... for some reason
        0u8.write_le(writer).unwrap();
    }

    // now write the data offsets
    writer.seek(SeekFrom::Start(data_offsets_pos)).unwrap();
    data_offsets.write(writer).unwrap();

    Ok(())
}

#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
#[brw(import(exh: &EXH))]
pub struct EXD {
    header: EXDHeader,

    #[br(count = header.index_size / core::mem::size_of::<ExcelDataOffset>() as u32)]
    #[bw(ignore)]
    data_offsets: Vec<ExcelDataOffset>,

    #[br(parse_with = parse_rows, args(&exh, &data_offsets))]
    #[bw(write_with = write_rows, args(&exh))]
    rows: Vec<ExcelRow>,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct ExcelSingleRow {
    pub columns: Vec<ColumnData>,
}

#[derive(Debug)]
pub enum ExcelRowKind {
    SingleRow(ExcelSingleRow),
    SubRows(Vec<ExcelSingleRow>)
}

#[derive(Debug)]
pub struct ExcelRow {
    pub row_id: u32,
    pub kind: ExcelRowKind,
}

impl EXD {
    pub fn from_existing(exh: &EXH, buffer: ByteSpan) -> Option<EXD> {
        EXD::read_args(&mut Cursor::new(&buffer), (exh,)).ok()
    }

    fn read_data_raw<T: Read + Seek, Z: BinRead<Args<'static> = ()>>(cursor: &mut T) -> Option<Z> {
        Z::read_options(cursor, Endian::Big, ()).ok()
    }

    fn read_column<T: Read + Seek>(
        cursor: &mut T,
        exh: &EXH,
        column: &ExcelColumnDefinition,
    ) -> Option<ColumnData> {
        let mut read_packed_bool = |shift: i32| -> bool {
            let bit = 1 << shift;
            let bool_data: i32 = Self::read_data_raw(cursor).unwrap_or(0);

            (bool_data & bit) == bit
        };

        match column.data_type {
            ColumnDataType::String => {
                let string_offset: u32 = Self::read_data_raw(cursor).unwrap();

                let old_pos = cursor.stream_position().unwrap();

                // -4 to take into account reading string_offset
                cursor
                    .seek(SeekFrom::Current(
                        (exh.header.data_offset as u32 + string_offset - 4).into(),
                    ))
                    .ok()?;

                let mut string = String::new();

                let mut byte: u8 = Self::read_data_raw(cursor).unwrap();
                while byte != 0 {
                    string.push(byte as char);
                    byte = Self::read_data_raw(cursor).unwrap();
                }

                cursor.seek(SeekFrom::Start(old_pos)).unwrap();

                Some(ColumnData::String(string))
            }
            ColumnDataType::Bool => {
                // FIXME: i believe Bool is int8?
                let bool_data: i32 = Self::read_data_raw(cursor).unwrap();

                Some(ColumnData::Bool(bool_data == 1))
            }
            ColumnDataType::Int8 => Some(ColumnData::Int8(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt8 => Some(ColumnData::UInt8(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Int16 => Some(ColumnData::Int16(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt16 => Some(ColumnData::UInt16(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Int32 => Some(ColumnData::Int32(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt32 => Some(ColumnData::UInt32(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Float32 => Some(ColumnData::Float32(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Int64 => Some(ColumnData::Int64(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt64 => Some(ColumnData::UInt64(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::PackedBool0 => Some(ColumnData::Bool(read_packed_bool(0))),
            ColumnDataType::PackedBool1 => Some(ColumnData::Bool(read_packed_bool(1))),
            ColumnDataType::PackedBool2 => Some(ColumnData::Bool(read_packed_bool(2))),
            ColumnDataType::PackedBool3 => Some(ColumnData::Bool(read_packed_bool(3))),
            ColumnDataType::PackedBool4 => Some(ColumnData::Bool(read_packed_bool(4))),
            ColumnDataType::PackedBool5 => Some(ColumnData::Bool(read_packed_bool(5))),
            ColumnDataType::PackedBool6 => Some(ColumnData::Bool(read_packed_bool(6))),
            ColumnDataType::PackedBool7 => Some(ColumnData::Bool(read_packed_bool(7))),
        }
    }

    fn write_data_raw<T: Write + Seek, Z: BinWrite<Args<'static> = ()>>(cursor: &mut T, value: &Z) {
        value.write_options(cursor, Endian::Big, ()).unwrap()
    }

     fn write_column<T: Write + Seek>(
        cursor: &mut T,
        column: &ColumnData,
        column_definition: &ExcelColumnDefinition,
     ) {
        let write_packed_bool = |cursor: &mut T, shift: i32, boolean: &bool| {
            let val = 0i32; // TODO
            Self::write_data_raw(cursor, &val);
        };

        match column {
            ColumnData::String(_) => {
                let string_offset = 0u32; // TODO, but 0 is fine for single string column data
                Self::write_data_raw(cursor, &string_offset);
            },
            ColumnData::Bool(val) => {
                match column_definition.data_type {
                    ColumnDataType::Bool => todo!(),
                    ColumnDataType::PackedBool0 => write_packed_bool(cursor, 0, val),
                    ColumnDataType::PackedBool1 => write_packed_bool(cursor, 1, val),
                    ColumnDataType::PackedBool2 => write_packed_bool(cursor, 2, val),
                    ColumnDataType::PackedBool3 => write_packed_bool(cursor, 3, val),
                    ColumnDataType::PackedBool4 => write_packed_bool(cursor, 4, val),
                    ColumnDataType::PackedBool5 => write_packed_bool(cursor, 5, val),
                    ColumnDataType::PackedBool6 => write_packed_bool(cursor, 6, val),
                    ColumnDataType::PackedBool7 => write_packed_bool(cursor, 7, val),
                    _ => panic!("This makes no sense!")
                }
            },
            ColumnData::Int8(val) => Self::write_data_raw(cursor, val),
            ColumnData::UInt8(val) => Self::write_data_raw(cursor, val),
            ColumnData::Int16(val) => Self::write_data_raw(cursor, val),
            ColumnData::UInt16(val) => Self::write_data_raw(cursor, val),
            ColumnData::Int32(val) => Self::write_data_raw(cursor, val),
            ColumnData::UInt32(val) => Self::write_data_raw(cursor, val),
            ColumnData::Float32(val) => Self::write_data_raw(cursor, val),
            ColumnData::Int64(val) => Self::write_data_raw(cursor, val),
            ColumnData::UInt64(val) => Self::write_data_raw(cursor, val),
        }
    }

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
                data_offset: 0,
                column_count: 0,
                page_count: 0,
                language_count: 0,
                row_count: 0,
            },
            column_definitions: vec![],
            pages: vec![],
            languages: vec![],
        };

        // Feeding it invalid data should not panic
        EXD::from_existing(&exh, &read(d).unwrap());
    }
}
