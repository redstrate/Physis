// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::identity_op)]
#![allow(unused_variables)] // for br(temp), meh

use std::io::SeekFrom;

use crate::common::Platform;
use crate::crc::Jamcrc;
use binrw::binrw;
use binrw::BinRead;

/// The type of this SqPack file.
#[binrw]
#[brw(repr = u8)]
enum SqPackFileType {
    /// Dat files.
    Data = 0x1,
    // Index/Index2 files.
    Index = 0x2,
}

#[binrw]
#[brw(magic = b"SqPack\0\0")]
pub struct SqPackHeader {
    platform_id: Platform,
    #[brw(pad_before = 3)]
    size: u32,
    version: u32,
    file_type: SqPackFileType,
}

#[binrw]
pub struct SqPackIndexHeader {
    size: u32,
    version: u32,
    index_data_offset: u32,
    index_data_size: u32,
    index_data_hash: [u8; 64],
    number_of_data_file: u32,
    synonym_data_offset: u32,
    synonym_data_size: u32,
    synonym_data_hash: [u8; 64],
    empty_block_data_offset: u32,
    empty_block_data_size: u32,
    empty_block_data_hash: [u8; 64],
    dir_index_data_offset: u32,
    dir_index_data_size: u32,
    dir_index_data_hash: [u8; 64],
    index_type: u32,
    #[br(pad_before = 656)]
    self_hash: [u8; 64],
}

#[binrw]
pub struct IndexHashTableEntry {
    pub hash: u64,

    #[br(temp)]
    #[bw(ignore)]
    data: u32,

    #[br(temp)]
    #[bw(ignore)]
    padding: u32,

    #[br(calc = (data & 0b1) == 0b1)]
    #[bw(ignore)]
    pub is_synonym: bool,

    #[br(calc = ((data & 0b1110) >> 1) as u8)]
    #[bw(ignore)]
    pub data_file_id: u8,

    #[br(calc = (data & !0xF) as u64 * 0x08)]
    #[bw(ignore)]
    pub offset: u64,
}

// The only difference between index and index2 is how the path hash is stored.
// The folder name and the filename are split in index1 (hence why it's 64-bits and not 32-bit)
// But in index2, its both the file and folder name in one single CRC hash.
#[binrw]
#[derive(Debug)]
pub struct Index2HashTableEntry {
    pub hash: u32,

    #[br(temp)]
    #[bw(ignore)]
    data: u32,

    #[br(calc = (data & 0b1) == 0b1)]
    #[bw(ignore)]
    pub is_synonym: bool,

    #[br(calc = ((data & 0b1110) >> 1) as u8)]
    #[bw(ignore)]
    pub data_file_id: u8,

    #[br(calc = (data & !0xF) as u64 * 0x08)]
    #[bw(ignore)]
    pub offset: u64,
}

#[derive(Debug)]
pub struct IndexEntry {
    pub hash: u64,
    pub data_file_id: u8,
    pub offset: u64,
}

#[binrw]
#[br(little)]
pub struct IndexFile {
    sqpack_header: SqPackHeader,

    #[br(seek_before = SeekFrom::Start(sqpack_header.size.into()))]
    index_header: SqPackIndexHeader,

    #[br(seek_before = SeekFrom::Start(index_header.index_data_offset.into()))]
    #[br(count = index_header.index_data_size / 16)]
    pub entries: Vec<IndexHashTableEntry>,
}

#[binrw]
#[br(little)]
pub struct Index2File {
    sqpack_header: SqPackHeader,

    #[br(seek_before = SeekFrom::Start(sqpack_header.size.into()))]
    index_header: SqPackIndexHeader,

    #[br(seek_before = SeekFrom::Start(index_header.index_data_offset.into()))]
    #[br(count = index_header.index_data_size / 8)]
    pub entries: Vec<Index2HashTableEntry>,
}

const CRC: Jamcrc = Jamcrc::new();

impl IndexFile {
    /// Creates a new reference to an existing index file.
    pub fn from_existing(path: &str) -> Option<Self> {
        let mut index_file = std::fs::File::open(path).ok()?;

        Self::read(&mut index_file).ok()
    }

    /// Calculates a partial hash for a given path
    pub fn calculate_partial_hash(path: &str) -> u32 {
        let lowercase = path.to_lowercase();

        CRC.checksum(lowercase.as_bytes())
    }

    /// Calculates a hash for `index` files from a game path.
    pub fn calculate_hash(path: &str) -> u64 {
        let lowercase = path.to_lowercase();

        if let Some(pos) = lowercase.rfind('/') {
            let (directory, filename) = lowercase.split_at(pos);

            let directory_crc = CRC.checksum(directory.as_bytes());
            let filename_crc = CRC.checksum(filename[1..filename.len()].as_bytes());

            (directory_crc as u64) << 32 | (filename_crc as u64)
        } else {
            CRC.checksum(lowercase.as_bytes()) as u64
        }
    }

    // TODO: turn into traits?
    pub fn exists(&self, path: &str) -> bool {
        let hash = IndexFile::calculate_hash(path);
        self.entries.iter().any(|s| s.hash == hash)
    }

    pub fn find_entry(&self, path: &str) -> Option<IndexEntry> {
        let hash = IndexFile::calculate_hash(path);

        if let Some(entry) = self.entries.iter().find(|s| s.hash == hash) {
            return Some(IndexEntry {
                hash: entry.hash,
                data_file_id: entry.data_file_id,
                offset: entry.offset,
            });
        }

        None
    }
}

impl Index2File {
    /// Creates a new reference to an existing index2 file.
    pub fn from_existing(path: &str) -> Option<Self> {
        let mut index_file = std::fs::File::open(path).ok()?;

        Self::read(&mut index_file).ok()
    }

    /// Calculates a hash for `index2` files from a game path.
    pub fn calculate_hash(path: &str) -> u32 {
        let lowercase = path.to_lowercase();

        CRC.checksum(lowercase.as_bytes())
    }

    pub fn exists(&self, path: &str) -> bool {
        let hash = Index2File::calculate_hash(path);
        self.entries.iter().any(|s| s.hash == hash)
    }

    pub fn find_entry(&self, path: &str) -> Option<IndexEntry> {
        let hash = Index2File::calculate_hash(path);

        if let Some(entry) = self.entries.iter().find(|s| s.hash == hash) {
            return Some(IndexEntry {
                hash: entry.hash as u64,
                data_file_id: entry.data_file_id,
                offset: entry.offset,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_index_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        IndexFile::from_existing(d.to_str().unwrap());
    }

    #[test]
    fn test_index2_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Index2File::from_existing(d.to_str().unwrap());
    }
}
