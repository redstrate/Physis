use std::io::{Cursor, Seek, SeekFrom};
use crate::gamedata::MemoryBuffer;
use binrw::{binread, Endian, ReadOptions};
use crate::common::Language;
use binrw::BinRead;
use crate::exh::{ColumnDataType, ExcelColumnDefinition, ExcelDataPagination, EXH};

#[binread]
#[br(magic = b"EXDF")]
#[br(big)]
struct EXDHeader {
    version : u16,

    #[br(pad_before = 2)]
    #[br(pad_after = 20)]
    index_size : u32
}

#[binread]
#[br(big)]
struct ExcelDataOffset {
    row_id : u32,
    pub offset : u32
}

#[binread]
#[br(big)]
struct ExcelDataRowHeader {
    data_size : u32,
    row_count : u16
}

#[binread]
#[br(big)]
pub struct EXD {
    header : EXDHeader,

    #[br(count = header.index_size / core::mem::size_of::<ExcelDataOffset>() as u32)]
    data_offsets : Vec<ExcelDataOffset>,

    #[br(ignore)]
    pub rows : Vec<ExcelRow>
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
    UInt64(u64)
}

pub struct ExcelRow {
    pub data : Vec<ColumnData>
}

impl EXD {
    fn read_data_raw<Z : BinRead>(cursor: &mut Cursor<&MemoryBuffer>) -> Option<Z> where <Z as BinRead>::Args: Default {
        Some(Z::read_options(cursor, &ReadOptions::new(Endian::Big) , <Z as BinRead>::Args::default()).unwrap())
    }

    fn read_column(cursor: &mut Cursor<&MemoryBuffer>, exh: &EXH, offset : u32, column : &ExcelColumnDefinition) -> Option<ColumnData> {
        let mut read_packed_bool = | shift : i32 | -> bool {
            let bit = 1 << shift;
            let bool_data : i32 = Self::read_data_raw(cursor).unwrap();

            (bool_data & bit) == bit
        };

        match column.data_type {
            ColumnDataType::String => {
                let string_offset : u32 = Self::read_data_raw(cursor).unwrap();

                cursor.seek(SeekFrom::Start((offset + exh.header.data_offset as u32 + string_offset).into())).ok()?;

                let mut string = String::new();

                let mut byte : u8 = Self::read_data_raw(cursor).unwrap();
                while byte != 0 {
                    string.push(byte as char);
                    byte = Self::read_data_raw(cursor).unwrap();
                }

                Some(ColumnData::String(string))
            }
            ColumnDataType::Bool => {
                // FIXME: i believe Bool is int8?
                let bool_data : i32 = Self::read_data_raw(cursor).unwrap();

                Some(ColumnData::Bool(bool_data == 1))
            }
            ColumnDataType::Int8 => {
                Some(ColumnData::Int8(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::UInt8 => {
                Some(ColumnData::UInt8(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::Int16 => {
                Some(ColumnData::Int16(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::UInt16 => {
                Some(ColumnData::UInt16(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::Int32 => {
                Some(ColumnData::Int32(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::UInt32 => {
                Some(ColumnData::UInt32(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::Float32 => {
                Some(ColumnData::Float32(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::Int64 => {
                Some(ColumnData::Int64(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::UInt64 => {
                Some(ColumnData::UInt64(Self::read_data_raw(cursor).unwrap()))
            }
            ColumnDataType::PackedBool0 => {
                Some(ColumnData::Bool(read_packed_bool(0)))
            }
            ColumnDataType::PackedBool1 => {
                Some(ColumnData::Bool(read_packed_bool(1)))
            }
            ColumnDataType::PackedBool2 => {
                Some(ColumnData::Bool(read_packed_bool(2)))
            }
            ColumnDataType::PackedBool3 => {
                Some(ColumnData::Bool(read_packed_bool(3)))
            }
            ColumnDataType::PackedBool4 => {
                Some(ColumnData::Bool(read_packed_bool(4)))
            }
            ColumnDataType::PackedBool5 => {
                Some(ColumnData::Bool(read_packed_bool(5)))
            }
            ColumnDataType::PackedBool6 => {
                Some(ColumnData::Bool(read_packed_bool(6)))
            }
            ColumnDataType::PackedBool7 => {
                Some(ColumnData::Bool(read_packed_bool(7)))
            }
        }
    }

    pub fn from_existing(exh : &EXH, buffer : &MemoryBuffer) -> Option<EXD> {
        let mut cursor = Cursor::new(buffer);
        let mut exd = EXD::read(&mut cursor).ok()?;

        for i in 0..exh.header.row_count {
            for offset in &exd.data_offsets {
                if offset.row_id == i {
                    cursor.seek(SeekFrom::Start(offset.offset.into())).ok()?;

                    let row_header = ExcelDataRowHeader::read(&mut cursor).ok()?;

                    let header_offset = offset.offset + 6;

                    let mut read_row = | offset : u32 | -> Option<ExcelRow> {
                        let mut subrow = ExcelRow { data : Vec::with_capacity(exh.column_definitions.len()) };

                        for column in &exh.column_definitions {
                            cursor.seek(SeekFrom::Start((offset + column.offset as u32).into())).ok()?;

                            subrow.data.push(Self::read_column(&mut cursor, exh, offset, column).unwrap());
                        }

                        Some(subrow)
                    };

                    if row_header.row_count > 1 {
                        for i in 0..row_header.row_count {
                            let subrow_offset = header_offset + (i * exh.header.data_offset + 2 * (i + 1)) as u32;

                            exd.rows.push(read_row(subrow_offset).unwrap());
                        }
                    } else {
                        exd.rows.push(read_row(header_offset).unwrap());
                    }
                }
            }
        }

        Some(exd)
    }

    pub fn calculate_filename(name : &str, language : Language, page : &ExcelDataPagination) -> String {
        use crate::common::get_language_code;

        return match language {
            Language::None => {
                format!("{name}_{}.exd", page.start_id)
            }
            lang => {
                format!("{name}_{}_{}.exd", page.start_id, get_language_code(&lang))
            }
        }
    }
}