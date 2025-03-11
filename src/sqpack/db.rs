// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use crate::common_file_operations::read_string;
use crate::common_file_operations::write_string;
use crate::sqpack::SqPackHeader;
use binrw::BinRead;
use binrw::binrw;
use binrw::helpers::until_eof;

#[binrw]
#[derive(Debug)]
pub struct SQDBHeader {
    size: u32,
    #[br(pad_after = 1016)] // nothing
    unk: u32,
}

// 264 bytes
#[binrw]
#[derive(Debug)]
pub struct SQDBEntry {
    #[br(pad_before = 4)] // 4 empty bytes
    offset: u32, // TODO: just a guess, offset into what?
    #[br(pad_after = 4)] // 4 more empty bytes
    size: u32, // TODO: also a guess

    // Corresponds to SplitPath hash
    filename_hash: u32,
    path_hash: u32,

    // TODO: this is terrible, just read until string nul terminator
    #[br(count = 240)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    path: String,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct SqPackDatabase {
    sqpack_header: SqPackHeader,

    header: SQDBHeader,

    #[br(parse_with = until_eof)]
    entries: Vec<SQDBEntry>,
}

impl SqPackDatabase {
    /// Reads an existing SQDB file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Self::read(&mut cursor).ok()
    }
}
