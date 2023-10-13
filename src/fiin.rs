// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::read;
use std::io::Cursor;

use binrw::{BinRead, BinWrite};
use binrw::binrw;
use crate::{ByteBuffer, ByteSpan};

use crate::sha1::Sha1;

#[binrw]
#[brw(magic = b"FileInfo")]
#[derive(Debug)]
#[brw(little)]
pub struct FileInfo {
    #[brw(pad_before = 16)]
    #[bw(calc = 1024)]
    _unknown: i32,

    #[br(temp)]
    #[bw(calc = (entries.len() * 96) as i32)]
    entries_size: i32,

    #[brw(pad_before = 992)]
    #[br(count = entries_size / 96)]
    pub entries: Vec<FIINEntry>,
}

#[binrw]
#[derive(Debug)]
pub struct FIINEntry {
    pub file_size: i32,

    #[brw(pad_before = 4)]
    #[br(count = 64)]
    #[bw(pad_size_to = 64)]
    #[bw(map = |x : &String | x.as_bytes())]
    #[br(map = | x: Vec<u8> | String::from_utf8(x).unwrap().trim_matches(char::from(0)).to_string())]
    pub file_name: String,

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
    /// The new FileInfo structure can then be serialized back into retail-compatible form.
    pub fn new(file_names: &[&str]) -> Option<FileInfo> {
        let mut entries = vec![];

        for name in file_names {
            let file = &read(name).expect("Cannot read file.");

            entries.push(FIINEntry {
                file_size: file.len() as i32,
                file_name: name.to_string(),
                sha1: Sha1::from(file).digest().bytes().to_vec(),
            });
        }

        Some(FileInfo { entries })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
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
}
