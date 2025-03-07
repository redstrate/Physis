// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::identity_op)]
#![allow(unused_variables)] // for br(temp), meh

use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use crate::common::Platform;
use crate::common::Region;
use crate::crc::Jamcrc;
use binrw::BinRead;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::Endian;
use binrw::Error;
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
#[derive(PartialEq, Debug)]
pub enum Hash {
    #[br(pre_assert(*index_type == IndexType::Index1))]
    SplitPath { name: u32, path: u32 },
    #[br(pre_assert(*index_type == IndexType::Index2))]
    FullPath(u32),
}

pub struct FileEntryData {
    pub is_synonym: bool,
    pub data_file_id: u8,
    pub offset: u64,
}

impl BinRead for FileEntryData {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> BinResult<Self> {
        let data = <u32>::read_options(reader, endian, ())?;
        Ok(Self {
            is_synonym: (data & 0b1) == 0b1,
            data_file_id: ((data & 0b1110) >> 1) as u8,
            offset: (data & !0xF) as u64 * 0x08,
        })
    }
}

impl BinWrite for FileEntryData {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> Result<(), Error> {
        // TODO: support synonym and data_file_id
        let data: u32 = self.offset.wrapping_div(0x08) as u32;

        data.write_options(writer, endian, ())
    }
}

#[binrw]
#[brw(import(index_type: &IndexType))]
pub struct FileEntry {
    #[br(args(index_type))]
    pub hash: Hash,

    pub data: FileEntryData,

    #[br(temp)]
    #[bw(calc = 0)]
    #[br(if(*index_type == IndexType::Index1))]
    padding: u32,
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
    #[bw(args(&index_header.index_type,))]
    pub entries: Vec<FileEntry>,

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
                data_file_id: entry.data.data_file_id,
                offset: entry.data.offset,
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, path::PathBuf};

    use binrw::BinWrite;

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
    fn readwrite_index1_file_entry() {
        let data = [
            0xEF, 0x02, 0x50, 0x1C, 0x68, 0xCF, 0x4E, 0x00, 0x60, 0x01, 0x6E, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let mut cursor = Cursor::new(&data);

        let file_entry =
            FileEntry::read_options(&mut cursor, Endian::Little, (&IndexType::Index1,)).unwrap();

        let expected_hash = Hash::SplitPath {
            name: 475005679,
            path: 5164904,
        };
        assert_eq!(file_entry.hash, expected_hash);
        assert_eq!(file_entry.data.is_synonym, false);
        assert_eq!(file_entry.data.data_file_id, 0);
        assert_eq!(file_entry.data.offset, 57674496);

        // Ensure if we write this it's identica'
        let mut new_data = Vec::new();
        {
            let mut write_cursor = Cursor::new(&mut new_data);
            file_entry
                .write_options(&mut write_cursor, Endian::Little, (&IndexType::Index1,))
                .unwrap();
        }

        assert_eq!(new_data, data);
    }
}
