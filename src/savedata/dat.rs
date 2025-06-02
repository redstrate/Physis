// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused_variables)] // just binrw things with br(temp)

use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub enum DatFileType {
    /// GEARSET.DAT
    #[brw(magic = 0x006d0005u32)]
    Gearset,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct DatHeader {
    pub file_type: DatFileType,
    pub max_size: u32,
    #[brw(pad_after = 4)] // empty bytes
    pub content_size: u32,
    #[br(temp)]
    #[bw(calc = 0xFF)]
    end_of_header: u8,
}
