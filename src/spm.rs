// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

/// Shader Parameter Map file, usually with the `.spm` file extension.
#[binrw]
#[derive(Debug, Clone)]
pub struct ShaderParameterMap {
    version: u32,
    column_count: u8,
    row_count: u8,
    columns_offset: u16,
    rows_offset: u16,
    values_offset: u16,

    #[br(count = column_count, seek_before = SeekFrom::Start((columns_offset as u64) << 2))]
    pub column_definitions: Vec<ColumnDefinition>,
    #[br(count = row_count, seek_before = SeekFrom::Start((rows_offset as u64) << 2))]
    pub row_definitions: Vec<RowDefinition>,
    #[br(count = row_count, seek_before = SeekFrom::Start((values_offset as u64) << 2), args { inner: (column_count,) })]
    pub rows: Vec<Row>,
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Clone)]
pub enum ColumnType {
    Float = 0,
    UInt = 1,
    Name = 2,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ColumnDefinition {
    /// CRC32
    pub name: u32,
    pub column_type: ColumnType,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct RowDefinition {
    /// CRC32
    pub table: u32,
    pub index: u32,
}

#[binrw]
#[derive(Debug, Clone)]
#[br(import(column_count: u8))]
pub struct Row {
    #[br(count = column_count)]
    pub columns: Vec<[u8; 4]>, // TODO: parse
}

impl ReadableFile for ShaderParameterMap {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(Self::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

impl WritableFile for ShaderParameterMap {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<ShaderParameterMap>();
    }
}
