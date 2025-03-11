// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Read, Seek, SeekFrom, Write};

use binrw::{BinRead, BinWrite, binrw};
use data::{BlockHeader, CompressionMode};

use crate::common::{Platform, Region};
use crate::compression::no_header_decompress;

mod data;
pub use data::SqPackData;

mod db;
pub use db::SqPackDatabase;

mod index;
pub use index::{IndexEntry, SqPackIndex};

/// The type of this SqPack file.
#[binrw]
#[brw(repr = u8)]
#[derive(Debug)]
pub(crate) enum SqPackFileType {
    /// FFXIV Explorer says "SQDB", whatever that is.
    SQDB = 0x0,
    /// Dat files.
    Data = 0x1,
    /// Index/Index2 files.
    Index = 0x2,
}

#[binrw]
#[brw(magic = b"SqPack\0\0")]
#[derive(Debug)]
pub(crate) struct SqPackHeader {
    #[brw(pad_size_to = 4)]
    platform_id: Platform,
    pub size: u32,
    // Have only seen version 1
    version: u32,
    #[brw(pad_size_to = 4)]
    file_type: SqPackFileType,

    // some unknown value, zeroed out for index files
    // XivAlexandar says date/time, where does that come from?
    unk1: u32,
    unk2: u32,

    #[br(pad_size_to = 4)]
    region: Region,

    #[brw(pad_before = 924)]
    #[brw(pad_after = 44)]
    // The SHA1 of the bytes immediately before this
    sha1_hash: [u8; 20],
}

pub(crate) fn read_data_block<T: Read + Seek>(
    mut buf: T,
    starting_position: u64,
) -> Option<Vec<u8>> {
    buf.seek(SeekFrom::Start(starting_position)).ok()?;

    let block_header = BlockHeader::read(&mut buf).unwrap();

    match block_header.compression {
        CompressionMode::Compressed {
            compressed_length,
            decompressed_length,
        } => {
            let mut compressed_data: Vec<u8> = vec![0; compressed_length as usize];
            buf.read_exact(&mut compressed_data).ok()?;

            let mut decompressed_data: Vec<u8> = vec![0; decompressed_length as usize];
            if !no_header_decompress(&mut compressed_data, &mut decompressed_data) {
                return None;
            }

            Some(decompressed_data)
        }
        CompressionMode::Uncompressed { file_size } => {
            let mut local_data: Vec<u8> = vec![0; file_size as usize];
            buf.read_exact(&mut local_data).ok()?;

            Some(local_data)
        }
    }
}

/// A fixed version of read_data_block accounting for differing compressed block sizes in ZiPatch files.
pub(crate) fn read_data_block_patch<T: Read + Seek>(mut buf: T) -> Option<Vec<u8>> {
    let block_header = BlockHeader::read(&mut buf).unwrap();

    match block_header.compression {
        CompressionMode::Compressed {
            compressed_length,
            decompressed_length,
        } => {
            let compressed_length: usize =
                ((compressed_length as usize + 143) & 0xFFFFFF80) - (block_header.size as usize);

            let mut compressed_data: Vec<u8> = vec![0; compressed_length];
            buf.read_exact(&mut compressed_data).ok()?;

            let mut decompressed_data: Vec<u8> = vec![0; decompressed_length as usize];
            if !no_header_decompress(&mut compressed_data, &mut decompressed_data) {
                return None;
            }

            Some(decompressed_data)
        }
        CompressionMode::Uncompressed { file_size } => {
            let new_file_size: usize = (file_size as usize + 143) & 0xFFFFFF80;

            let mut local_data: Vec<u8> = vec![0; file_size as usize];
            buf.read_exact(&mut local_data).ok()?;

            buf.seek(SeekFrom::Current(
                (new_file_size - block_header.size as usize - file_size as usize) as i64,
            ))
            .ok()?;

            Some(local_data)
        }
    }
}

pub(crate) fn write_data_block_patch<T: Write + Seek>(mut writer: T, data: Vec<u8>) {
    let new_file_size: usize = (data.len() + 143) & 0xFFFFFF80;

    // This only adds uncompressed data for now, to simplify implementation
    // TODO: write compressed blocks
    let block_header = BlockHeader {
        size: (new_file_size - data.len()) as u32, // TODO: i have no idea what this value is from
        compression: CompressionMode::Uncompressed {
            file_size: data.len() as i32,
        },
    };
    block_header.write(&mut writer).unwrap();

    data.write(&mut writer).unwrap();
}
