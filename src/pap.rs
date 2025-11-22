// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteSpan;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
enum SkeletonType {
    #[brw(magic = 0u8)]
    Human,
    #[brw(magic = 1u8)]
    Monster,
    #[brw(magic = 2u8)]
    DemiHuman,
    #[brw(magic = 3u8)]
    Weapon,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct PapHeader {
    magic: i32, // TODO: what magic?
    version: i32,

    num_animations: i16,
    model_id: u16,
    model_type: SkeletonType,
    variant: i32,

    info_offset: i32,
    havok_position: i32,
    footer_position: i32,
}

#[derive(Debug)]
pub struct Pap {}

impl Pap {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        PapHeader::read(&mut cursor).ok()?;

        Some(Pap {})
    }
}
