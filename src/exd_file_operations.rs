// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom, Write},
};

use binrw::{BinRead, BinResult, BinWrite, Endian};

use crate::{
    exd::{
        ColumnData, DataSection, EXD, EXDHeader, ExcelDataOffset, ExcelRow, ExcelRowKind,
        ExcelSingleRow,
    },
    exh::{ColumnDataType, EXH, ExcelColumnDefinition},
};

#[binrw::parser(reader)]
pub fn read_data_sections(header: &EXDHeader) -> BinResult<Vec<DataSection>> {
    let mut rows = Vec::new();

    // we have to do this annoying thing because they specified it in bytes,
    // not an actual count of data sections
    let begin_pos = reader.stream_position().unwrap();
    loop {
        let current_pos = reader.stream_position().unwrap();
        if current_pos - begin_pos >= header.data_section_size as u64 {
            break;
        }

        let data_section = DataSection::read_be(reader).unwrap();
        rows.push(data_section);
    }

    Ok(rows)
}

#[binrw::parser(reader)]
pub fn parse_rows(exh: &EXH, data_offsets: &Vec<ExcelDataOffset>) -> BinResult<Vec<ExcelRow>> {
    let mut rows = Vec::new();

    for offset in data_offsets {
        reader.seek(SeekFrom::Start(offset.offset.into()))?;

        // TODO: use DataSection here
        let _size: u32 = u32::read_be(reader).unwrap();
        let row_count: u16 = u16::read_be(reader).unwrap();
        //let row_header = DataSection::read(reader)?;

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
                    .push(EXD::read_column(reader, exh, row_offset, column).unwrap());
            }

            Some(subrow)
        };

        let new_row = if row_count > 1 {
            let mut rows = Vec::new();
            for i in 0..row_count {
                let subrow_offset = data_offset + (i * exh.header.data_offset + 2 * (i + 1)) as u32;

                rows.push(read_row(subrow_offset).unwrap());
            }
            ExcelRowKind::SubRows(rows)
        } else {
            ExcelRowKind::SingleRow(read_row(data_offset).unwrap())
        };
        rows.push(ExcelRow {
            row_id: offset.row_id,
            kind: new_row,
        });
    }

    Ok(rows)
}

#[binrw::writer(writer)]
pub fn write_rows(rows: &Vec<ExcelRow>, exh: &EXH) -> BinResult<()> {
    // seek past the data offsets, which we will write later
    let data_offsets_pos = writer.stream_position().unwrap();
    writer
        .seek(SeekFrom::Current(
            (core::mem::size_of::<ExcelDataOffset>() * rows.len()) as i64,
        ))
        .unwrap();

    let mut data_offsets = Vec::new();

    for row in rows {
        data_offsets.push(ExcelDataOffset {
            row_id: row.row_id,
            offset: writer.stream_position().unwrap() as u32,
        });

        // skip row header for now, because we don't know the size yet!
        let row_header_pos = writer.stream_position().unwrap();

        writer.seek(SeekFrom::Current(6)).unwrap(); // u32 + u16

        let old_pos = writer.stream_position().unwrap();

        // write column data
        {
            let mut write_row = |row: &ExcelSingleRow| {
                let mut column_definitions: Vec<(ExcelColumnDefinition, ColumnData)> = exh
                    .column_definitions
                    .clone()
                    .into_iter()
                    .zip(row.columns.clone().into_iter())
                    .collect::<Vec<_>>();

                // we need to sort them by offset
                column_definitions.sort_by(|(a, _), (b, _)| a.offset.cmp(&b.offset));

                // handle packed bools
                let mut packed_bools: HashMap<u16, u8> = HashMap::new();

                let mut write_packed_bool =
                    |definition: &ExcelColumnDefinition, shift: i32, boolean: &bool| {
                        if !packed_bools.contains_key(&definition.offset) {
                            packed_bools.insert(definition.offset, 0u8);
                        }

                        if *boolean {
                            let bit = 1 << shift;
                            *packed_bools.get_mut(&definition.offset).unwrap() |= bit;
                        }
                    };

                // process packed bools before continuing, since we need to know what their final byte form is
                for (definition, column) in &column_definitions {
                    match &column {
                        ColumnData::Bool(val) => match definition.data_type {
                            ColumnDataType::PackedBool0 => write_packed_bool(definition, 0, val),
                            ColumnDataType::PackedBool1 => write_packed_bool(definition, 1, val),
                            ColumnDataType::PackedBool2 => write_packed_bool(definition, 2, val),
                            ColumnDataType::PackedBool3 => write_packed_bool(definition, 3, val),
                            ColumnDataType::PackedBool4 => write_packed_bool(definition, 4, val),
                            ColumnDataType::PackedBool5 => write_packed_bool(definition, 5, val),
                            ColumnDataType::PackedBool6 => write_packed_bool(definition, 6, val),
                            ColumnDataType::PackedBool7 => write_packed_bool(definition, 7, val),
                            _ => {} // not relevant
                        },
                        _ => {} // not relevant
                    }
                }

                let mut strings_len = 0;
                for (definition, column) in &column_definitions {
                    EXD::write_column(
                        writer,
                        column,
                        definition,
                        &mut strings_len,
                        &mut packed_bools,
                    );

                    // TODO: temporary workaround until i can figure out why it has 4 extra bytes in test_write's case
                    if definition.data_type == ColumnDataType::Int8 && column_definitions.len() == 1
                    {
                        0u32.write_le(writer).unwrap();
                    }
                }
            };

            match &row.kind {
                ExcelRowKind::SingleRow(excel_single_row) => write_row(excel_single_row),
                ExcelRowKind::SubRows(excel_single_rows) => {
                    for row in excel_single_rows {
                        write_row(row);
                    }
                }
            }
        }

        // write strings at the end of column data
        {
            let mut write_row_strings = |row: &ExcelSingleRow| {
                for column in &row.columns {
                    if let ColumnData::String(val) = column {
                        let bytes = val.as_bytes();
                        bytes.write(writer).unwrap();

                        // nul terminator
                        0u8.write_le(writer).unwrap();
                    }
                }
            };

            match &row.kind {
                ExcelRowKind::SingleRow(excel_single_row) => write_row_strings(excel_single_row),
                ExcelRowKind::SubRows(excel_single_rows) => {
                    for row in excel_single_rows {
                        write_row_strings(row);
                    }
                }
            }
        }

        // aligned to the next 4 byte boundary
        let boundary_pos = writer.stream_position().unwrap();
        let remainder = boundary_pos.div_ceil(4) * 4;
        for _ in 0..remainder - boundary_pos {
            0u8.write_le(writer).unwrap();
        }

        let new_pos = writer.stream_position().unwrap();

        // write row header
        writer.seek(SeekFrom::Start(row_header_pos)).unwrap();

        let row_header = DataSection {
            size: (new_pos - old_pos) as u32,
            row_count: 1, // TODO: hardcoded
        };
        row_header.write(writer).unwrap();

        // restore pos
        writer.seek(SeekFrom::Start(new_pos)).unwrap();
    }

    // now write the data offsets
    writer.seek(SeekFrom::Start(data_offsets_pos)).unwrap();
    data_offsets.write(writer).unwrap();

    Ok(())
}

impl EXD {
    fn read_data_raw<T: Read + Seek, Z: BinRead<Args<'static> = ()>>(cursor: &mut T) -> Option<Z> {
        Z::read_options(cursor, Endian::Big, ()).ok()
    }

    pub(crate) fn read_column<T: Read + Seek>(
        cursor: &mut T,
        exh: &EXH,
        row_offset: u32,
        column: &ExcelColumnDefinition,
    ) -> Option<ColumnData> {
        let mut read_packed_bool = |shift: i32| -> bool {
            let bit = 1 << shift;
            let bool_data: u8 = Self::read_data_raw(cursor).unwrap_or(0);

            (bool_data & bit) == bit
        };

        match column.data_type {
            ColumnDataType::String => {
                let string_offset: u32 = Self::read_data_raw(cursor).unwrap();

                cursor
                    .seek(SeekFrom::Start(
                        (row_offset + exh.header.data_offset as u32 + string_offset).into(),
                    ))
                    .ok()?;

                let mut string = String::new();

                let mut byte: u8 = Self::read_data_raw(cursor).unwrap();
                while byte != 0 {
                    string.push(byte as char);
                    byte = Self::read_data_raw(cursor).unwrap();
                }

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
            ColumnDataType::UInt16 => {
                Some(ColumnData::UInt16(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::Int32 => Some(ColumnData::Int32(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt32 => {
                Some(ColumnData::UInt32(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::Float32 => {
                Some(ColumnData::Float32(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::Int64 => Some(ColumnData::Int64(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt64 => {
                Some(ColumnData::UInt64(Self::read_data_raw(cursor).unwrap()))
            }
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

    pub(crate) fn write_column<T: Write + Seek>(
        cursor: &mut T,
        column: &ColumnData,
        column_definition: &ExcelColumnDefinition,
        strings_len: &mut u32,
        packed_bools: &mut HashMap<u16, u8>,
    ) {
        match column {
            ColumnData::String(val) => {
                let string_offset = *strings_len;
                Self::write_data_raw(cursor, &string_offset);
                *strings_len += val.len() as u32 + 1;
            }
            ColumnData::Bool(_) => match column_definition.data_type {
                ColumnDataType::Bool => todo!(),
                // packed bools are handled in write_rows
                ColumnDataType::PackedBool0
                | ColumnDataType::PackedBool1
                | ColumnDataType::PackedBool2
                | ColumnDataType::PackedBool3
                | ColumnDataType::PackedBool4
                | ColumnDataType::PackedBool5
                | ColumnDataType::PackedBool6
                | ColumnDataType::PackedBool7 => {
                    if let Some(byte) = packed_bools.get(&column_definition.offset) {
                        byte.write_le(cursor).unwrap();

                        // then remove it so the next packed bool column doesn't write it again
                        packed_bools.remove(&column_definition.offset);
                    }
                }
                _ => panic!("This makes no sense!"),
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
}
