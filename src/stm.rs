// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use crate::ByteSpan;
use binrw::BinRead;
use binrw::{BinReaderExt, binrw};

/// Maximum number of elements in one row
const MAX_ELEMENTS: usize = 128;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct StmHeader {
    #[br(pad_before = 4)] // TODO: what is this byte?
    entry_count: i32,

    #[br(count = entry_count)]
    keys: Vec<u16>,

    #[br(count = entry_count)]
    offsets: Vec<u16>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DyePack {
    diffuse: [f32; 3],
    specular: [f32; 3],
    emissive: [f32; 3],
    gloss: f32,
    specular_power: f32,
}

#[derive(Debug)]
pub struct StainingTemplate {}

impl StainingTemplate {
    /// Reads an existing ULD file
    pub fn from_existing(buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let header = StmHeader::read(&mut cursor).unwrap();

        for entry_offset in header.offsets {
            let offset = entry_offset as i32 * 2 + 8 + 4 * header.entry_count;

            // read the stm entry
            cursor.seek(SeekFrom::Start(offset as u64)).ok()?;

            // read the value offsets
            let mut ends = [0u16; 5];
            for end in &mut ends {
                *end = cursor.read_le::<u16>().unwrap() * 2;
            }

            /*let new_offset = (offset + 10) as u64;

            let diffuse_entries = StainingTemplate::read_array::<Half3>(&mut cursor, new_offset, ends[0] as usize);
            let specular_entries = StainingTemplate::read_array::<Half3>(&mut cursor, new_offset + ends[0] as u64, ends[1] as usize - ends[0] as usize);
            let emissive_entries = StainingTemplate::read_array::<Half3>(&mut cursor, new_offset + ends[1] as u64, ends[2] as usize - ends[1] as usize);
            let gloss_entries = StainingTemplate::read_array::<Half1>(&mut cursor, new_offset + ends[2] as u64, ends[3] as usize - ends[2] as usize);
            let specular_power_entries = StainingTemplate::read_array::<Half1>(&mut cursor, new_offset + ends[3] as u64, ends[4] as usize - ends[3] as usize);*/
        }

        Some(StainingTemplate {})
    }

    #[allow(unused)]
    fn read_array<T: binrw::BinRead<Args<'static> = ()> + Default + Clone + Copy>(
        cursor: &mut Cursor<ByteSpan>,
        offset: u64,
        size: usize,
    ) -> Vec<T> {
        cursor.seek(SeekFrom::Start(offset)).unwrap();

        let array_size = size / std::mem::size_of::<T>();
        if array_size == 0 {
            vec![T::default(); MAX_ELEMENTS]
        } else if array_size == 1 {
            let element = cursor.read_le::<T>().unwrap();
            vec![element; MAX_ELEMENTS]
        } else if array_size < MAX_ELEMENTS {
            let real_count = array_size - MAX_ELEMENTS / std::mem::size_of::<T>();
            let mut values = vec![];
            let mut indices = vec![];
            values.push(T::default());
            for _ in 0..real_count {
                values.push(cursor.read_le::<T>().unwrap());
            }

            let eof_marker = cursor.read_le::<u8>().unwrap();
            assert_eq!(eof_marker, 0xFF);

            for _ in 0..MAX_ELEMENTS {
                indices.push(cursor.read_le::<u8>().unwrap());
            }

            let mut vec = vec![];
            for index in indices {
                if (index as usize) < values.len() {
                    vec.push(values[index as usize]);
                } else {
                    vec.push(T::default());
                }
            }

            vec
        } else if array_size == MAX_ELEMENTS {
            let mut vec = vec![];
            for _ in 0..size {
                vec.push(cursor.read_le::<T>().unwrap());
            }
            vec
        } else {
            panic!("Too many elements");
        }
    }
}
