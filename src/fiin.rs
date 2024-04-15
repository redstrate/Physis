// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::read;
use std::io::Cursor;
use std::path::Path;

use binrw::{BinRead, BinWrite};
use binrw::binrw;
use crate::{ByteBuffer, ByteSpan};

use crate::sha1::Sha1;

#[binrw]
#[brw(magic = b"FileInfo")]
#[derive(Debug)]
#[brw(little)]
/// File info, which contains SHA1 of one or more files
pub struct FileInfo {
    #[brw(pad_before = 16)]
    #[bw(calc = 1024)]
    _unknown: i32,

    #[br(temp)]
    #[bw(calc = (entries.len() * 96) as i32)]
    entries_size: i32,

    #[brw(pad_before = 992)]
    #[br(count = entries_size / 96)]
    /// File info entries
    pub entries: Vec<FIINEntry>,
}

#[binrw]
#[derive(Debug)]
/// A file info entry
pub struct FIINEntry {
    /// File size (in bytes)
    pub file_size: i32,

    /// The file name
    #[brw(pad_before = 4)]
    #[br(count = 64)]
    #[bw(pad_size_to = 64)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    pub file_name: String,

    /// SHA1 of the file
    #[br(count = 24)]
    #[bw(pad_size_to = 24)]
    pub sha1: Vec<u8>,
}

impl FileInfo {
    /// Parses an existing FIIN file.
    pub fn from_existing(buffer: ByteSpan) -> Option<FileInfo> {
        let mut cursor = Cursor::new(buffer);
        FileInfo::read(&mut cursor).ok()
    }

    /// Writes file info into a new file
    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write(&mut cursor).ok()?;
        }

        Some(buffer)
    }

    /// Creates a new FileInfo structure from a list of filenames. These filenames must be present in
    /// the current working directory in order to be read properly, since it also generates SHA1
    /// hashes.
    /// 
    /// These paths are converted to just their filenames.
    ///
    /// The new FileInfo structure can then be serialized back into retail-compatible form.
    pub fn new(files: &[&str]) -> Option<FileInfo> {
        let mut entries = vec![];

        for path in files {
            let file = &read(path).expect("Cannot read file.");

            entries.push(FIINEntry {
                file_size: file.len() as i32,
                file_name: Path::new(path).file_name()?.to_str()?.to_string(),
                sha1: Sha1::from(file).digest().bytes().to_vec(),
            });
        }

        Some(FileInfo { entries })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{File, read};
    use std::path::PathBuf;

    use crate::fiin::FileInfo;

    fn common_setup() -> FileInfo {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("test.fiin");

        FileInfo::from_existing(&read(d).unwrap()).unwrap()
    }

    #[test]
    fn basic_parsing() {
        let fiin = common_setup();

        assert_eq!(fiin.entries[0].file_name, "test.txt");

        assert_eq!(fiin.entries[1].file_name, "test.exl");
    }

    #[test]
    fn basic_writing() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("test.fiin");

        let valid_fiin = &read(d).unwrap();

        let mut d2 = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d2.push("resources/tests");
        d2.push("test.txt");

        let mut d3 = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d3.push("resources/tests");
        d3.push("test.exl");

        let testing_fiin = FileInfo::new(&[
            d2.to_str().unwrap(),
            d3.to_str().unwrap()
        ]).unwrap();

        assert_eq!(*valid_fiin, testing_fiin.write_to_buffer().unwrap());
    }
}
