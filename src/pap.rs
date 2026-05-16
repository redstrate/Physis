// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::common_file_operations::read_bool_from;
use crate::common_file_operations::read_string;
use crate::common_file_operations::write_bool_as;
use crate::common_file_operations::write_string;
use crate::string_heap::StringHeap;
use crate::tmb::Tmb;
use binrw::BinRead;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
pub enum SkeletonType {
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
pub struct PapAnimation {
    #[br(count = 32)]
    #[bw(pad_size_to = 32)]
    #[bw(map = write_string)]
    #[br(map = read_string)]
    pub name: String,
    animation_type: u16,
    havok_index: i16,
    #[br(map = read_bool_from::<i32>)]
    #[bw(map = write_bool_as::<i32>)]
    face: bool,
}

#[binrw::parser(reader, endian)]
fn read_tmbs(num_animations: i16, start_position: u64) -> BinResult<Vec<Tmb>> {
    reader.seek(SeekFrom::Start(start_position))?;

    let mut tmbs = Vec::new();

    // Because of weirdness + string heaps we need to skip padding and find each TMLB manually.
    loop {
        if tmbs.len() >= num_animations as usize {
            break;
        }

        let magic = <[u8; 4]>::read_options(reader, endian, ())?;
        if &magic == b"TMLB" {
            reader.seek(SeekFrom::Current(-4))?;

            let string_heap = StringHeap::from(-8); // FIXME: a hack i guess
            tmbs.push(Tmb::read_options(reader, endian, (&string_heap,)).unwrap());
        }
    }

    Ok(tmbs)
}

#[binrw]
#[brw(magic = b"pap ")]
#[derive(Debug)]
pub struct Pap {
    version: i32,

    num_animations: i16,
    model_id: u16,
    pub model_type: SkeletonType,
    variant: u8,

    info_offset: i32,
    havok_position: i32,
    tmb_offset: i32,

    #[br(count = num_animations)]
    pub animations: Vec<PapAnimation>,

    // TODO: remove seek_before, we can probably read it linearly
    #[br(seek_before = SeekFrom::Start(havok_position as u64), count = tmb_offset - havok_position, restore_position)]
    havok_data: Vec<u8>,

    #[br(parse_with = read_tmbs, args(num_animations, tmb_offset as u64))]
    #[bw(ignore)] // TODO: support writing
    pub tmbs: Vec<Tmb>,
}

impl ReadableFile for Pap {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        Pap::read_options(&mut cursor, platform.endianness(), ()).ok()
    }
}

impl WritableFile for Pap {
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())
                .ok()?;
        }

        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Pap>();
    }
}
