// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::Cursor;

use binrw::BinRead;
use binrw::binrw;
use binrw::helpers::until_eof;

use crate::common::{Language, Platform};
use crate::exh::ExcelDataPagination;
use crate::{ByteSpan, ReadableFile};

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

impl EXDHeader {
    pub const SIZE: usize = 0x20;
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

/// Excel data file, usually with the `.exd` file extension.
///
/// Represents a page in an Excel Sheet. You most likely want to use the types in the `excel` module.
#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug)]
pub struct EXD {
    pub(crate) header: EXDHeader,

    #[br(count = header.data_offset_size / core::mem::size_of::<ExcelDataOffset>() as u32)]
    #[bw(ignore)]
    pub(crate) data_offsets: Vec<ExcelDataOffset>,

    #[br(parse_with = until_eof)]
    #[bw(ignore)]
    pub remaining_data: Vec<u8>,
}

impl EXD {
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
}

impl ReadableFile for EXD {
    fn from_existing(_platform: Platform, buffer: ByteSpan) -> Option<EXD> {
        EXD::read_args(&mut Cursor::new(&buffer), ()).ok()
    }
}

// For more complex tests, see `excel.rs`.
#[cfg(test)]
mod tests {
    use crate::exh::{EXH, EXHHeader, SheetRowKind};
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
                row_kind: SheetRowKind::SingleRow,
                unk2: 0,
                unk3: 0,
            },
            column_definitions: vec![],
            pages: vec![],
            languages: vec![],
        };

        // Feeding it invalid data should not panic
        EXD::from_existing(Platform::Win32, &read(d).unwrap());
    }
}
