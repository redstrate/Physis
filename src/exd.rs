// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use std::io::Cursor;

use binrw::BinRead;
use binrw::binrw;

use crate::common::{Language, Platform};
use crate::exh::ExcelDataPagination;
use crate::{ByteSpan, ReadableFile};

#[binrw]
#[brw(magic = b"EXDF")]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
/// Represents a page in an Excel Sheet. You most likely want to use the types in the [excel](crate::excel) module.
#[binrw]
#[brw(big)]
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EXD {
    pub(crate) header: EXDHeader,

    #[br(count = header.data_offset_size / core::mem::size_of::<ExcelDataOffset>() as u32)]
    #[bw(ignore)]
    pub(crate) data_offsets: Vec<ExcelDataOffset>,

    #[br(count = header.data_section_size)]
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
        match language {
            Language::None => {
                format!("{name}_{}.exd", page.start_id)
            }
            lang => {
                format!("{name}_{}_{}.exd", page.start_id, lang.shortname())
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
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<EXD>();
    }
}
