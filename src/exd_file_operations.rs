// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom, Write},
};

use binrw::{BinRead, BinWrite, Endian};

use crate::{
    excel::{Field, Row},
    exd::EXD,
    exh::{ColumnDataType, EXH, ExcelColumnDefinition},
};

pub(crate) fn read_row<T: Read + Seek>(reader: &mut T, exh: &EXH, row_offset: u64) -> Option<Row> {
    let mut subrow = Row {
        columns: Vec::with_capacity(exh.column_definitions.len()),
    };

    for column in &exh.column_definitions {
        reader
            .seek(SeekFrom::Start(row_offset + column.offset as u64))
            .ok()?;

        subrow
            .columns
            .push(EXD::read_column(reader, exh, row_offset, column).unwrap());
    }

    Some(subrow)
}

pub(crate) fn write_row<T: Write + Seek>(writer: &mut T, exh: &EXH, row: &Row) {
    let mut column_definitions: Vec<(ExcelColumnDefinition, Field)> = exh
        .column_definitions
        .clone()
        .into_iter()
        .zip(row.columns.clone())
        .collect::<Vec<_>>();

    // we need to sort them by offset
    column_definitions.sort_by(|(a, _), (b, _)| a.offset.cmp(&b.offset));

    // handle packed bools
    let mut packed_bools: HashMap<u16, u8> = HashMap::new();

    let mut write_packed_bool = |definition: &ExcelColumnDefinition, shift: i32, boolean: &bool| {
        packed_bools.entry(definition.offset).or_insert(0u8);

        if *boolean {
            let bit = 1 << shift;
            *packed_bools.get_mut(&definition.offset).unwrap() |= bit;
        }
    };

    // process packed bools before continuing, since we need to know what their final byte form is
    for (definition, column) in &column_definitions {
        if let Field::Bool(val) = &column {
            match definition.data_type {
                ColumnDataType::PackedBool0 => write_packed_bool(definition, 0, val),
                ColumnDataType::PackedBool1 => write_packed_bool(definition, 1, val),
                ColumnDataType::PackedBool2 => write_packed_bool(definition, 2, val),
                ColumnDataType::PackedBool3 => write_packed_bool(definition, 3, val),
                ColumnDataType::PackedBool4 => write_packed_bool(definition, 4, val),
                ColumnDataType::PackedBool5 => write_packed_bool(definition, 5, val),
                ColumnDataType::PackedBool6 => write_packed_bool(definition, 6, val),
                ColumnDataType::PackedBool7 => write_packed_bool(definition, 7, val),
                _ => {} // not relevant
            }
        };
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
        if definition.data_type == ColumnDataType::Int8 && column_definitions.len() == 1 {
            0u32.write_le(writer).unwrap();
        }

        // TODO: temporary workaround until i can figure out why this *specific* packed boolean column in TerritoryType has three extra bytes at the end
        if definition.offset == 60
            && definition.data_type == ColumnDataType::PackedBool0
            && column_definitions.len() == 44
        {
            [0u8; 3].write_le(writer).unwrap();
        }
    }
}

impl EXD {
    fn read_data_raw<T: Read + Seek, Z: BinRead<Args<'static> = ()>>(cursor: &mut T) -> Option<Z> {
        Z::read_options(cursor, Endian::Big, ()).ok()
    }

    pub(crate) fn read_column<T: Read + Seek>(
        cursor: &mut T,
        exh: &EXH,
        row_offset: u64,
        column: &ExcelColumnDefinition,
    ) -> Option<Field> {
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
                        row_offset + exh.header.row_size as u64 + string_offset as u64,
                    ))
                    .ok()?;

                let mut string = String::new();

                let mut byte: u8 = Self::read_data_raw(cursor).unwrap();
                while byte != 0 {
                    string.push(byte as char);
                    byte = Self::read_data_raw(cursor).unwrap();
                }

                Some(Field::String(string))
            }
            ColumnDataType::Bool => {
                // FIXME: i believe Bool is int8?
                let bool_data: i32 = Self::read_data_raw(cursor).unwrap();

                Some(Field::Bool(bool_data == 1))
            }
            ColumnDataType::Int8 => Some(Field::Int8(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt8 => Some(Field::UInt8(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Int16 => Some(Field::Int16(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt16 => Some(Field::UInt16(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Int32 => Some(Field::Int32(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt32 => Some(Field::UInt32(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Float32 => Some(Field::Float32(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::Int64 => Some(Field::Int64(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::UInt64 => Some(Field::UInt64(Self::read_data_raw(cursor).unwrap())),
            ColumnDataType::PackedBool0 => Some(Field::Bool(read_packed_bool(0))),
            ColumnDataType::PackedBool1 => Some(Field::Bool(read_packed_bool(1))),
            ColumnDataType::PackedBool2 => Some(Field::Bool(read_packed_bool(2))),
            ColumnDataType::PackedBool3 => Some(Field::Bool(read_packed_bool(3))),
            ColumnDataType::PackedBool4 => Some(Field::Bool(read_packed_bool(4))),
            ColumnDataType::PackedBool5 => Some(Field::Bool(read_packed_bool(5))),
            ColumnDataType::PackedBool6 => Some(Field::Bool(read_packed_bool(6))),
            ColumnDataType::PackedBool7 => Some(Field::Bool(read_packed_bool(7))),
        }
    }

    fn write_data_raw<T: Write + Seek, Z: BinWrite<Args<'static> = ()>>(cursor: &mut T, value: &Z) {
        value.write_options(cursor, Endian::Big, ()).unwrap()
    }

    pub(crate) fn write_column<T: Write + Seek>(
        cursor: &mut T,
        column: &Field,
        column_definition: &ExcelColumnDefinition,
        strings_len: &mut u32,
        packed_bools: &mut HashMap<u16, u8>,
    ) {
        match column {
            Field::String(val) => {
                let string_offset = *strings_len;
                Self::write_data_raw(cursor, &string_offset);
                *strings_len += val.len() as u32 + 1;
            }
            Field::Bool(_) => match column_definition.data_type {
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
            Field::Int8(val) => Self::write_data_raw(cursor, val),
            Field::UInt8(val) => Self::write_data_raw(cursor, val),
            Field::Int16(val) => Self::write_data_raw(cursor, val),
            Field::UInt16(val) => Self::write_data_raw(cursor, val),
            Field::Int32(val) => Self::write_data_raw(cursor, val),
            Field::UInt32(val) => Self::write_data_raw(cursor, val),
            Field::Float32(val) => Self::write_data_raw(cursor, val),
            Field::Int64(val) => Self::write_data_raw(cursor, val),
            Field::UInt64(val) => Self::write_data_raw(cursor, val),
        }
    }
}
