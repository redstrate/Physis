// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::read;
use std::io::Cursor;
use std::path::Path;

use crate::common_file_operations::{read_string, write_string};
use crate::{ByteBuffer, ByteSpan, Platform, ReadableFile, WritableFile};
use binrw::binrw;
use binrw::{BinRead, BinWrite};

#[binrw]
#[brw(magic = b"FileInfo")]
#[derive(Debug)]
#[brw(little)]
/// File info file, usually with the `.fiin` file extension.
///
/// Purpose unknown, but contains SHA1 information of various executable-related files.
pub struct FileInfo {
    #[brw(pad_before = 16)]
    #[bw(calc = 1024)]
    _unknown: i32,

    #[br(temp)]
    #[bw(calc = (entries.len() * 96) as i32)]
    entries_size: i32,

    #[brw(pad_before = 992)]
    #[br(count = entries_size / 96)]
    /// File info entries.
    pub entries: Vec<FIINEntry>,
}

#[binrw]
#[derive(Debug)]
/// An entry in a FIIN file.
pub struct FIINEntry {
    /// File size (in bytes).
    pub file_size: i32,

    /// The file name.
    #[brw(pad_before = 4)]
    #[br(count = 64)]
    #[bw(pad_size_to = 64)]
    #[bw(map = write_string)]
    #[br(map = read_string)]
    pub file_name: String,

    /// SHA1 of the file.
    #[br(count = 24)]
    #[bw(pad_size_to = 24)]
    pub sha1: Vec<u8>,
}

impl ReadableFile for FileInfo {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(Self::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

impl WritableFile for FileInfo {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

impl FileInfo {
    /// Creates a new FileInfo structure from a list of filenames. These filenames must be present in
    /// the current working directory in order to be read properly, since it also generates SHA1
    /// hashes.
    ///
    /// These paths are converted to just their filenames.
    pub fn new(files: &[(&str, Vec<u8>)]) -> Option<Self> {
        let mut entries = vec![];

        for (path, sha1) in files {
            let file = &read(path).expect("Cannot read file.");

            entries.push(FIINEntry {
                file_size: file.len() as i32,
                file_name: Path::new(path).file_name()?.to_str()?.to_string(),
                sha1: sha1.clone(),
            });
        }

        Some(Self { entries })
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use crate::fiin::FileInfo;
    use crate::{Platform, ReadableFile, WritableFile, pass_random_invalid};

    fn common_setup() -> FileInfo {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("test.fiin");

        FileInfo::from_existing(Platform::Win32, &read(d).unwrap()).unwrap()
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
            (
                d2.to_str().unwrap(),
                vec![
                    0x69, 0x34, 0x2c, 0x5c, 0x39, 0xe5, 0xae, 0x5f, 0x00, 0x77, 0xae, 0xcc, 0x32,
                    0xc0, 0xf8, 0x18, 0x11, 0xfb, 0x81, 0x93, 0x00, 0x00, 0x00, 0x00,
                ],
            ),
            (
                d3.to_str().unwrap(),
                vec![
                    0x12, 0xf9, 0xa2, 0x2d, 0x99, 0x96, 0x71, 0x2d, 0x4a, 0xd7, 0xa3, 0xc9, 0x14,
                    0xaf, 0xe6, 0x14, 0x5e, 0xfa, 0x93, 0x83, 0x00, 0x00, 0x00, 0x00,
                ],
            ),
        ])
        .unwrap();

        assert_eq!(
            *valid_fiin,
            testing_fiin.write_to_buffer(Platform::Win32).unwrap()
        );
    }

    #[test]
    fn test_invalid() {
        pass_random_invalid::<FileInfo>();
    }
}
