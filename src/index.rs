use binrw::binrw;
use binrw::BinRead;
use bitfield_struct::bitfield;
use std::io::SeekFrom;

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq)]
enum PlatformId {
    Windows,
    PS3,
    PS4,
}

#[binrw]
#[br(magic = b"SqPack")]
pub struct SqPackHeader {
    #[br(pad_before = 2)]
    platform_id: PlatformId,
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

#[bitfield(u32)]
#[binrw]
#[br(map = | x: u32 | Self::from(x))]
pub struct IndexHashBitfield {
    #[bits(1)]
    pub size: u32,

    #[bits(3)]
    pub data_file_id: u32,

    #[bits(28)]
    pub offset: u32,
}

#[binrw]
pub struct IndexHashTableEntry {
    pub(crate) hash: u64,
    #[br(pad_after = 4)]
    pub(crate) bitfield: IndexHashBitfield,
}

#[derive(Debug)]
pub struct IndexEntry {
    pub hash: u64,
    pub data_file_id: u8,
    pub offset: u32,
}

#[binrw]
pub struct IndexFile {
    sqpack_header: SqPackHeader,

    #[br(seek_before = SeekFrom::Start(sqpack_header.size.into()))]
    index_header: SqPackIndexHeader,

    #[br(seek_before = SeekFrom::Start(index_header.index_data_offset.into()))]
    #[br(count = index_header.index_data_size / 16)]
    pub entries: Vec<IndexHashTableEntry>,
}

impl IndexFile {
    /// Creates a new reference to an existing index file.
    pub fn from_existing(path: &str) -> Option<IndexFile> {
        let mut index_file = std::fs::File::open(path).ok()?;

        IndexFile::read(&mut index_file).ok()
    }
}
