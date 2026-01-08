// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{ByteBuffer, ByteSpan, Platform, ReadableFile, WritableFile};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};

/// Excel list file, usually with the `.exl` file extension.
///
/// Contains a list of every Excel sheet available in-game.
#[derive(Debug)]
pub struct EXL {
    /// The version of the list.
    pub version: i32,

    /// The entries of the list.
    pub entries: Vec<(String, i32)>,
}

impl ReadableFile for EXL {
    fn from_existing(_platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut exl = Self {
            version: 0,
            entries: Vec::new(),
        };

        let cursor = Cursor::new(buffer);
        let reader = BufReader::new(cursor);

        for line in reader.lines().map_while(Result::ok) {
            if let Some((name, value)) = line.split_once(',')
                && let Ok(parsed_value) = value.parse()
            {
                if name == "EXLT" {
                    exl.version = parsed_value;
                } else if !name.starts_with('#') {
                    // Ignore rows with comments
                    exl.entries.push((name.to_string(), parsed_value));
                }
            }
        }

        Some(exl)
    }
}

impl WritableFile for EXL {
    fn write_to_buffer(&self, _platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            writer
                .write_all(format!("EXLT,{}", self.version).as_ref())
                .ok()?;

            for (key, value) in &self.entries {
                writer.write_all(format!("\n{key},{value}").as_ref()).ok()?;
            }
        }

        Some(buffer)
    }
}

impl EXL {
    /// Checks whether or not the list contains `key`.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::fs::read;
    /// # use std::path::PathBuf;
    /// # use physis::exl::EXL;
    /// # use physis::Platform;
    /// # use physis::ReadableFile;
    /// # let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    /// # d.push("resources/tests");
    /// # d.push("test.exl");
    /// # let exl_file = read(d).unwrap();
    /// let exl = EXL::from_existing(Platform::Win32, &exl_file).unwrap();
    /// exl.contains("Foo");
    /// ```
    pub fn contains(&self, key: &str) -> bool {
        self.entries.iter().any(|t| t.0 == key)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use crate::pass_random_invalid;

    use super::*;

    fn common_setup() -> EXL {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("test.exl");

        EXL::from_existing(Platform::Win32, &read(d).unwrap()).unwrap()
    }

    #[test]
    fn test_invalid() {
        pass_random_invalid::<EXL>();
    }

    #[test]
    fn version_parsing() {
        let exl = common_setup();

        assert_eq!(exl.version, 2);
    }

    #[test]
    fn contains() {
        let exl = common_setup();

        assert!(exl.contains("Foo"));

        // should be case-sensitive
        assert!(!exl.contains("foo"));
    }

    #[test]
    fn test_write() {
        let existing_exl = common_setup();

        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("test.exl");

        let exl = read(d).unwrap();

        let mut out = std::io::stdout();
        out.write_all(&existing_exl.write_to_buffer(Platform::Win32).unwrap())
            .unwrap();
        out.flush().unwrap();

        assert_eq!(existing_exl.write_to_buffer(Platform::Win32).unwrap(), exl);
    }
}
