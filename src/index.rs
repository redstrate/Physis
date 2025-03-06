// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::identity_op)]
#![allow(unused_variables)] // for br(temp), meh

use std::io::SeekFrom;

use crate::common::Platform;
use crate::common::Region;
use crate::crc::Jamcrc;
use binrw::BinRead;
use binrw::binrw;

/// The type of this SqPack file.
#[binrw]
#[brw(repr = u8)]
enum SqPackFileType {
    /// FFXIV Explorer says "SQDB", whatever that is.
    SQDB = 0x0,
    /// Dat files.
    Data = 0x1,
    /// Index/Index2 files.
    Index = 0x2,
}

#[binrw]
#[brw(magic = b"SqPack\0\0")]
pub struct SqPackHeader {
    #[brw(pad_size_to = 4)]
    platform_id: Platform,
    size: u32,
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

#[binrw]
#[derive(Debug)]
pub struct SegementDescriptor {
    count: u32,
    offset: u32,
    size: u32,
    #[brw(pad_after = 40)]
    sha1_hash: [u8; 20],
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
pub enum IndexType {
    Index1,
    Index2,
}

#[binrw]
#[derive(Debug)]
pub struct SqPackIndexHeader {
    size: u32,

    #[brw(pad_after = 4)]
    file_descriptor: SegementDescriptor,

    // Count in this descriptor correlates to the number of dat files.
    data_descriptor: SegementDescriptor,

    unknown_descriptor: SegementDescriptor,

    folder_descriptor: SegementDescriptor,

    #[brw(pad_size_to = 4)]
    pub(crate) index_type: IndexType,

    #[brw(pad_before = 656)]
    #[brw(pad_after = 44)]
    // The SHA1 of the bytes immediately before this
    sha1_hash: [u8; 20],
}

#[binrw]
#[br(import(index_type: &IndexType))]
#[derive(PartialEq)]
pub enum Hash {
    #[br(pre_assert(*index_type == IndexType::Index1))]
    SplitPath { name: u32, path: u32 },
    #[br(pre_assert(*index_type == IndexType::Index2))]
    FullPath(u32),
}

#[binrw]
#[br(import(index_type: &IndexType))]
pub struct IndexHashTableEntry {
    #[br(args(index_type))]
    pub hash: Hash,

    #[br(temp)]
    #[bw(ignore)]
    data: u32,

    #[br(temp)]
    #[bw(calc = 0)]
    #[br(if(*index_type == IndexType::Index1))]
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

#[binrw]
#[derive(Debug)]
pub struct DataEntry {
    // A bunch of 0xFFFFFFFF
    unk: [u8; 256],
}

#[binrw]
#[derive(Debug)]
pub struct FolderEntry {
    hash: u32,
    files_offset: u32,
    // Divide by 0x10 to get the number of files
    #[brw(pad_after = 4)]
    total_files_size: u32,
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

    #[br(seek_before = SeekFrom::Start(index_header.file_descriptor.offset.into()), count = index_header.file_descriptor.size / 16, args { inner: (&index_header.index_type,) })]
    pub entries: Vec<IndexHashTableEntry>,

    #[br(seek_before = SeekFrom::Start(index_header.data_descriptor.offset.into()))]
    #[br(count = index_header.data_descriptor.size / 256)]
    pub data_entries: Vec<DataEntry>,

    /*#[br(seek_before = SeekFrom::Start(index_header.unknown_descriptor.offset.into()))]
    #[br(count = index_header.unknown_descriptor.size / 16)]
    pub unknown_entries: Vec<IndexHashTableEntry>,*/
    #[br(seek_before = SeekFrom::Start(index_header.folder_descriptor.offset.into()))]
    #[br(count = index_header.folder_descriptor.size / 16)]
    pub folder_entries: Vec<FolderEntry>,
}

const CRC: Jamcrc = Jamcrc::new();

impl IndexFile {
    /// Creates a new reference to an existing index file.
    pub fn from_existing(path: &str) -> Option<Self> {
        let mut index_file = std::fs::File::open(path).ok()?;

        println!("Reading {}!", path);

        Self::read(&mut index_file).ok()
    }

    /// Calculates a partial hash for a given path
    pub fn calculate_partial_hash(path: &str) -> u32 {
        let lowercase = path.to_lowercase();

        CRC.checksum(lowercase.as_bytes())
    }

    /// Calculates a hash for `index` files from a game path.
    pub fn calculate_hash(&self, path: &str) -> Hash {
        let lowercase = path.to_lowercase();

        return match &self.index_header.index_type {
            IndexType::Index1 => {
                if let Some(pos) = lowercase.rfind('/') {
                    let (directory, filename) = lowercase.split_at(pos);

                    let directory_crc = CRC.checksum(directory.as_bytes());
                    let filename_crc = CRC.checksum(filename[1..filename.len()].as_bytes());

                    Hash::SplitPath {
                        name: filename_crc,
                        path: directory_crc,
                    }
                } else {
                    // TODO: is this ever hit?
                    panic!("This is unexpected, why is the file sitting outside of a folder?");
                }
            }
            IndexType::Index2 => Hash::FullPath(CRC.checksum(lowercase.as_bytes())),
        };
    }

    pub fn exists(&self, path: &str) -> bool {
        let hash = self.calculate_hash(path);
        self.entries.iter().any(|s| s.hash == hash)
    }

    pub fn find_entry(&self, path: &str) -> Option<IndexEntry> {
        let hash = self.calculate_hash(path);

        if let Some(entry) = self.entries.iter().find(|s| s.hash == hash) {
            let full_hash = match hash {
                Hash::SplitPath { name, path } => (path as u64) << 32 | (name as u64),
                Hash::FullPath(hash) => hash as u64,
            };
            return Some(IndexEntry {
                hash: 0,
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
}
