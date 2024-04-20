// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::identity_op)]

use std::io::SeekFrom;

use crate::common::Platform;
use crate::crc::Jamcrc;
use binrw::binrw;
use binrw::BinRead;
use modular_bitfield::prelude::*;

#[binrw]
#[br(magic = b"SqPack")]
pub struct SqPackHeader {
    #[br(pad_before = 2)]
    platform_id: Platform,
    #[br(pad_before = 3)]
    size: u32,
    version: u32,
    file_type: u32,
}

#[binrw]
pub struct SqPackIndexHeader {
    size: u32,
    file_type: u32,
    index_data_offset: u32,
    index_data_size: u32,
}

#[bitfield]
#[binrw]
#[br(map = Self::from_bytes)]
#[derive(Clone, Copy, Debug)]
pub struct IndexHashBitfield {
    pub size: B1,
    pub data_file_id: B3,
    pub offset: B28,
}

#[binrw]
pub struct IndexHashTableEntry {
    pub hash: u64,
    #[br(pad_after = 4)]
    pub(crate) bitfield: IndexHashBitfield,
}

// The only difference between index and index2 is how the path hash is stored.
// The folder name and the filename are split in index1 (hence why it's 64-bits and not 32-bit)
// But in index2, its both the file and folder name in one single CRC hash.
#[binrw]
#[derive(Debug)]
pub struct Index2HashTableEntry {
    pub hash: u32,
    pub(crate) bitfield: IndexHashBitfield,
}

#[derive(Debug)]
pub struct IndexEntry {
    pub hash: u64,
    pub data_file_id: u8,
    pub offset: u32,
}

#[binrw]
#[br(little)]
pub struct IndexFile {
    sqpack_header: SqPackHeader,

    #[br(seek_before = SeekFrom::Start(sqpack_header.size.into()))]
    index_header: SqPackIndexHeader,

    #[br(seek_before = SeekFrom::Start(index_header.index_data_offset.into()))]
    // +4 because of padding
    #[br(count = index_header.index_data_size / core::mem::size_of::<IndexHashTableEntry>() as u32 + 4)]
    pub entries: Vec<IndexHashTableEntry>,
}

#[binrw]
#[br(little)]
pub struct Index2File {
    sqpack_header: SqPackHeader,

    #[br(seek_before = SeekFrom::Start(sqpack_header.size.into()))]
    index_header: SqPackIndexHeader,

    #[br(seek_before = SeekFrom::Start(index_header.index_data_offset.into()))]
    #[br(count = index_header.index_data_size / core::mem::size_of::<Index2HashTableEntry>() as u32)]
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

    pub fn find_entry(&self, path: &str) -> Option<&IndexHashTableEntry> {
        let hash = IndexFile::calculate_hash(path);
        self.entries.iter().find(|s| s.hash == hash)
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

    pub fn find_entry(&self, path: &str) -> Option<&Index2HashTableEntry> {
        let hash = Index2File::calculate_hash(path);
        self.entries.iter().find(|s| s.hash == hash)
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
