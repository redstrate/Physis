use crate::gamedata::MemoryBuffer;
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Cursor};

/// Represents an Excel List.
pub struct EXL {
    /// The version of the list.
    pub version: i32,

    /// The entries of the list.
    pub entries: HashMap<String, i32>,
}

impl EXL {
    /// Initializes `EXL` from an existing list.
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<EXL> {
        let mut exl = Self {
            version: 0,
            entries: HashMap::new(),
        };

        let cursor = Cursor::new(buffer);
        let reader = BufReader::new(cursor);

        for (_, line) in reader.lines().enumerate() {
            // now parse the line!

            let unwrap = line.unwrap();
            let (name, value) = unwrap.split_once(',').unwrap();

            let parsed_value: i32 = value.parse().unwrap();

            if name == "EXLT" {
                exl.version = parsed_value;
            } else {
                exl.entries.insert(name.parse().unwrap(), parsed_value);
            }
        }

        Some(exl)
    }

    /// Checks whether or not the list contains a key.
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
        self.entries.contains_key(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read;
    use std::path::PathBuf;

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
}
