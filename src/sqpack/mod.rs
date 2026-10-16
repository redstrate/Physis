// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Read, Seek, SeekFrom, Write};

use binrw::{BinRead, BinResult, BinWrite, Endian, binrw};
use data::{BlockHeader, CompressionMode};
use zlib_rs::{InflateConfig, ReturnCode, decompress_slice};

use crate::common::Platform;

mod data;
pub use data::SqPackData;

mod db;
pub use db::{SqPackDatabase, SqPackDatabaseEntry};

mod index;
pub use index::{FileEntry, FileEntryData, Hash, IndexType, SqPackIndex};

/// The type of this SqPack file.
#[binrw]
#[brw(repr = u32)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum SqPackFileType {
    /// [SqPackDatabase] files.
    Database = 0x0,
    /// [SqPackData] files.
    Data = 0x1,
    /// [SqPackIndex] files.
    Index = 0x2,
}

#[binrw]
#[brw(magic = b"SqPack\0\0")]
#[derive(Debug, Clone)]
pub(crate) struct SqPackHeader {
    /// The platform meant to read this file.
    #[brw(pad_size_to = 4)]
    platform: Platform,
    /// The size of this header, in bytes.
    pub size: u32,
    /// Have only seen version 1.
    version: u32,
    /// What kind of file this is.
    file_type: SqPackFileType,

    // TODO: some unknown value, zeroed out for index files
    // XivAlexandar says date/time, where does that come from?
    unk1: u32,
    unk2: u32,

    // TODO: this is possibly region, but CN users reported this as 0 so maybe not?
    region: u8,

    #[brw(pad_before = 924)]
    #[brw(pad_after = 44)]
    /// The SHA1 of the bytes immediately before this.
    sha1_hash: [u8; 20],
}

pub(crate) fn read_data_block<T: Read + Seek>(
    buf: &mut T,
    endianness: Endian,
    starting_position: u64,
) -> crate::Result<Vec<u8>> {
    buf.seek(SeekFrom::Start(starting_position))?;

    let block_header = BlockHeader::read_options(buf, endianness, ())?;

    match block_header.compression {
        CompressionMode::Compressed {
            compressed_length,
            decompressed_length,
        } => {
            let mut compressed_data: Vec<u8> = vec![0; compressed_length as usize];
            buf.read_exact(&mut compressed_data)?;

            let mut decompressed_data: Vec<u8> = vec![0; decompressed_length as usize];
            let (output, err) = decompress_slice(
                &mut decompressed_data,
                &compressed_data,
                InflateConfig { window_bits: -15 },
            );
            if err != ReturnCode::Ok {
                return Err(crate::Error::Zlib(err as i32));
            }

            Ok(output.to_vec())
        }
        CompressionMode::Uncompressed { file_size } => {
            let mut local_data: Vec<u8> = vec![0; file_size as usize];
            buf.read_exact(&mut local_data)?;

            Ok(local_data)
        }
    }
}

/// A fixed version of read_data_block accounting for differing compressed block sizes in ZiPatch files.
pub(crate) fn read_data_block_patch<T: Read + Seek>(
    buf: &mut T,
    endianness: Endian,
) -> BinResult<Vec<u8>> {
    let block_header = BlockHeader::read_options(buf, endianness, ())?;

    match block_header.compression {
        CompressionMode::Compressed {
            compressed_length,
            decompressed_length,
        } => {
            let compressed_length: usize =
                ((compressed_length as usize + 143) & 0xFFFFFF80) - (block_header.size as usize);

            let mut compressed_data: Vec<u8> = vec![0; compressed_length];
            buf.read_exact(&mut compressed_data)?;

            let mut decompressed_data: Vec<u8> = vec![0; decompressed_length as usize];
            let (output, err) = decompress_slice(
                &mut decompressed_data,
                &compressed_data,
                InflateConfig { window_bits: -15 },
            );
            if err != ReturnCode::Ok {
                return Err(binrw::Error::Custom {
                    pos: 0,
                    err: Box::new(crate::Error::Zlib(err as i32)),
                });
            }

            Ok(output.to_vec())
        }
        CompressionMode::Uncompressed { file_size } => {
            let new_file_size: usize = (file_size as usize + 143) & 0xFFFFFF80;

            let mut local_data: Vec<u8> = vec![0; file_size as usize];
            buf.read_exact(&mut local_data)?;

            buf.seek(SeekFrom::Current(
                (new_file_size - block_header.size as usize - file_size as usize) as i64,
            ))?;

            Ok(local_data)
        }
    }
}

pub(crate) fn write_data_block_patch<T: Write + Seek>(
    writer: &mut T,
    endianness: Endian,
    data: Vec<u8>,
) -> crate::Result<()> {
    let new_file_size: usize = (data.len() + 143) & 0xFFFFFF80;

    // This only adds uncompressed data for now, to simplify implementation
    // TODO: write compressed blocks
    let block_header = BlockHeader {
        size: (new_file_size - data.len()) as u32, // TODO: i have no idea what this value is from
        compression: CompressionMode::Uncompressed {
            file_size: data.len() as i32,
        },
    };
    block_header.write_options(writer, endianness, ())?;

    data.write(writer)?;

    Ok(())
}
