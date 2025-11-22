// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{ByteBuffer, ByteSpan};
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};

/// Excel list file, usually with the `.exl` file extension.
///
/// Contains a list of every Excel sheet available in-game.
pub struct EXL {
    /// The version of the list.
    pub version: i32,

    /// The entries of the list.
    pub entries: Vec<(String, i32)>,
}

impl EXL {
    /// Initializes `EXL` from an existing list.
    pub fn from_existing(buffer: ByteSpan) -> Option<EXL> {
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

    /// Writes data back to a buffer.
    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
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

    /// Checks whether or not the list contains `key`.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::fs::read;
    /// # use std::path::PathBuf;
    /// # use physis::exl::EXL;
    /// # let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    /// # d.push("resources/tests");
    /// # d.push("test.exl");
    /// # let exl_file = read(d).unwrap();
    /// let exl = EXL::from_existing(&exl_file).unwrap();
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

    use super::*;

    fn common_setup() -> EXL {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("test.exl");

        EXL::from_existing(&read(d).unwrap()).unwrap()
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
        out.write_all(&existing_exl.write_to_buffer().unwrap())
            .unwrap();
        out.flush().unwrap();

        assert_eq!(existing_exl.write_to_buffer().unwrap(), exl);
    }

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        EXL::from_existing(&read(d).unwrap());
    }
}
