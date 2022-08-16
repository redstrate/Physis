use std::fs::read;
use std::io::Cursor;
use binrw::binrw;
use crate::gamedata::MemoryBuffer;
use binrw::BinRead;

#[binrw]
#[brw(magic = b"FileInfo")]
#[derive(Debug)]
pub struct FileInfo {
    #[brw(pad_before = 16)]
    #[br(ignore)]
    #[bw(calc = 1024)]
    unknown : i32,

    #[br(temp)]
    #[bw(calc = (entries.len() * 96) as i32)]
    entries_size : i32,

    #[brw(pad_before = 992)]
    #[br(count = entries_size / 96)]
    entries : Vec<FIINEntry>
}

#[binrw]
#[derive(Debug)]
pub struct FIINEntry {
    file_size : i32,

    #[brw(pad_before = 4)]
    #[br(count = 64)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    #[bw(map = | x : &String | x.as_bytes().to_vec())]
    #[bw(pad_size_to = 64)]
    file_name: String,

    #[br(count = 24)]
    #[bw(pad_size_to = 24)]
    sha1 : Vec<u8>
}

impl FileInfo {
    /// Parses an existing FIIN file.
    pub fn from_existing(buffer : &MemoryBuffer) -> Option<FileInfo> {
        let mut cursor = Cursor::new(buffer);
        FileInfo::read(&mut cursor).ok()
    }

    /// Creates a new FileInfo structure from a list of filenames. These filenames must be present in
    /// the current working directory in order to be read properly, since it also generates SHA1
    /// hashes.
    ///
    /// The new FileInfo structure can then be serialized back into retail-compatible form.
    pub fn new(file_names : Vec<&str>) -> Option<FileInfo> {
        let mut entries = vec![];

        for name in file_names {
            let file = &read(&name).expect("Cannot read file.");
            
            entries.push(FIINEntry {
                file_size: file.len() as i32,
                file_name: name.to_string(),
                sha1: sha1_smol::Sha1::from(file).digest().bytes().to_vec()
            });
        }

        Some(FileInfo {
            entries
        })
    }
}