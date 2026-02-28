// SPDX-FileCopyrightText: 2026 Kaze
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;
use std::io::Write;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::string_heap::HeapPointer;
use crate::string_heap::HeapStringFromPointer;
use crate::string_heap::StringHeap;
use binrw::BinRead;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::binrw;

/// Environment binary file, usually with the `.envb` file extension.
#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[brw(magic = b"ENVB")]
pub struct Envb {
    /// Size of the file, including this header.
    file_size: u32,
    envs_count: u32,

    #[br(count = envs_count, args { inner: (string_heap,) })]
    #[bw(write_with = write_envs, args(&mut string_heap,))]
    pub envs: Vec<EnvsHeader>,
}

#[binrw::writer(writer, endian)]
pub(crate) fn write_envs(envs: &Vec<EnvsHeader>, string_heap: &mut StringHeap) -> BinResult<()> {
    for env in envs {
        env.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

#[binrw]
#[brw(magic = b"ENVS")]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct EnvsHeader {
    /// Size of this header, in bytes. Should be the same as `EnvsHeader::SIZE`.
    size: u32,
    /// Always 6.
    pub version: u32,
    /// Offset to the sections array.
    offset_to_sections: u32,
    /// Number of sections.
    section_count: u32,
    /// Seems to indicate the remaining amount of bytes in this file, including this u32.
    remaining_size: u32,
    unk1: u32,

    /// List of children sections.
    #[br(count = section_count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset_to_sections as i64 - EnvsHeader::SIZE as i64 + 4))] // Read starting from version
    #[br(restore_position)]
    #[bw(write_with = write_child_sections, args(&mut string_heap,))]
    pub sections: Vec<EnvChildSection>,
}

#[binrw::writer(writer, endian)]
pub(crate) fn write_child_sections(
    sections: &Vec<EnvChildSection>,
    string_heap: &mut StringHeap,
) -> BinResult<()> {
    for section in sections {
        section.write_options(writer, endian, (string_heap,))?;
    }

    Ok(())
}

impl EnvsHeader {
    pub const SIZE: usize = 0x18;
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct EnvChildSection {
    offset: u32, // 0xc in size
    count: u32,
    unk3: u32,
    offset_to_floats: u32,

    #[br(count = count, args { inner: (string_heap,) })]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub unknown1: Vec<EnvUnknown1>,

    #[br(seek_before = SeekFrom::Current(offset_to_floats as i64 - EnvChildSection::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    floats: [f32; 5],
}

impl EnvChildSection {
    pub const SIZE: usize = 0x10;
}

#[binrw::parser(reader)]
fn unknown2_from_offsets(
    offsets: &Vec<u32>,
    string_heap: &StringHeap,
) -> BinResult<Vec<EnvUnknown2>> {
    let base_offset = reader.stream_position()?;

    let mut layers: Vec<EnvUnknown2> = vec![];

    for offset in offsets {
        let layer_offset = *offset as u64;

        reader.seek(SeekFrom::Start(base_offset + layer_offset))?;
        layers.push(EnvUnknown2::read_le_args(reader, (string_heap,))?); // TODO: don't assume LE
    }

    Ok(layers)
}

#[binrw]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Debug)]
pub struct EnvUnknown1 {
    offset: u32,
    count: u32,
    /// Up to 14, I think.
    index: u32,

    #[br(count = count)]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvUnknown1::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub unknown2_offsets: Vec<u32>,

    // TODO: This is bad, we are failing to parse part of bgcommon/env/local/ffxiv_lenv/lenv_sea_s1/lenv_s1_cave.envb.
    // This is just a workaround.
    #[br(try, parse_with = unknown2_from_offsets, args(&unknown2_offsets, string_heap))]
    #[br(seek_before = SeekFrom::Current(offset as i64 - EnvUnknown1::SIZE as i64))]
    #[br(restore_position)]
    #[bw(ignore)] // TODO: support writing
    pub unknown2: Vec<EnvUnknown2>,
}

impl EnvUnknown1 {
    pub const SIZE: usize = 0xc;
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap))]
#[bw(import(string_heap: &mut StringHeap))]
pub struct EnvUnknown2 {
    // unconfirmed
    pub unk1: f32,
    pub unk2: f32,
    pub unk3: f32,
    pub unk4: f32,
    pub unk5: f32,
    pub unk6: f32,
    pub unk7: f32,
    pub unk8: f32,

    pub unk9: f32,
    pub unk10: f32,
    pub unk11: f32,
    pub unk12: f32,
    #[br(temp)]
    #[bw(ignore)]
    heap_pointer: HeapPointer,
    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub name: HeapStringFromPointer,
    #[br(args(heap_pointer, string_heap))]
    #[bw(args(string_heap))]
    pub name2: HeapStringFromPointer,
    pub unk15: f32,
    pub unk16: f32,
    pub unk17: f32,
    pub unk18: f32,
    pub unk19: f32,
    pub unk20: f32,
    pub unk21: f32,
    pub unk22: f32,
    pub unk23: f32,
    pub unk24: f32,
}

impl EnvUnknown2 {
    pub const SIZE: usize = 0x20; // unconfirmed
}

#[binrw]
#[derive(Debug)]
pub struct Ending {
    // NOTE: not always 0x14 and 0x14! Some files have 0x3E instead.
    // string heap is somewhere in here for some reason
    unk1: [u8; 13],
}

impl ReadableFile for Envb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);
        let string_heap = StringHeap::from(cursor.position());

        Envb::read_options(&mut cursor, endianness, (&string_heap,)).ok()
    }
}

impl WritableFile for Envb {
    fn write_to_buffer(&self, platform: Platform) -> Option<crate::ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut string_heap = StringHeap::from(0);

            // TODO: need dual pass

            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), (&mut string_heap,))
                .ok()?;

            string_heap
                .write_options(&mut cursor, platform.endianness(), ())
                .ok()?;

            let unk_ending = &[0x0; 8];
            cursor.write_all(unk_ending).ok()?;
        }

        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::read, path::PathBuf};

    use super::*;

    #[test]
    fn read_empty_envb() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("lenv_s1h1_outdoor.envb");

        let envb = Envb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(envb.envs.len(), 1);

        let envs = &envb.envs[0];
        assert_eq!(envs.section_count, 0);
    }

    #[test]
    fn write_empty_envb() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("lenv_s1h1_outdoor.envb");

        let envb_bytes = read(d).unwrap();
        let env = Envb::from_existing(Platform::Win32, &envb_bytes).unwrap();

        assert_eq!(env.write_to_buffer(Platform::Win32).unwrap(), envb_bytes);
    }
}
