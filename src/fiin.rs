use std::ffi::CString;
use std::io::Cursor;
use binrw::binread;
use crate::gamedata::MemoryBuffer;
use binrw::BinRead;

#[binread]
#[br(magic = b"FileInfo")]
#[derive(Debug)]
pub struct FileInfo {
    #[br(pad_before = 20)]
    #[br(temp)]
    entries_size : i32,

    #[br(pad_before = 992)]
    #[br(count = entries_size / 96)]
    entries : Vec<FIINEntry>
}

#[binread]
#[derive(Debug)]
pub struct FIINEntry {
    file_size : i32,

    #[br(pad_before = 4)]
    #[br(count = 64)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    file_name: String,

    #[br(count = 24)]
    sha1 : Vec<u8>
}

impl FileInfo {
    pub fn from_existing(buffer : &MemoryBuffer) -> Option<FileInfo> {
        let mut cursor = Cursor::new(buffer);
        Some(FileInfo::read(&mut cursor).ok()?)
    }
}