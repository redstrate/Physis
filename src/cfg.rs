// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{ByteBuffer, ByteSpan};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};

/// Represents a collection of keys, mapped to their values.
#[derive(Debug)]
pub struct ConfigMap {
    /// A map of setting name to value.
    pub keys: Vec<(String, String)>,
}

/// Represents a config file, which is made up of categories and settings. Categories may have zero to one setting.
#[derive(Debug)]
pub struct ConfigFile {
    /// The categories present in this config file.
    pub categories: Vec<String>,
    /// A mapping of category to keys.
    pub settings: HashMap<String, ConfigMap>,
}

impl ConfigFile {
    /// Parses an existing config file.
    pub fn from_existing(buffer: ByteSpan) -> Option<ConfigFile> {
        let mut cfg = ConfigFile {
            categories: Vec::new(),
            settings: HashMap::new(),
        };

        let cursor = Cursor::new(buffer);
        let reader = BufReader::new(cursor);

        let mut current_category: Option<String> = None;

        for line in reader.lines().map_while(Result::ok) {
            if !line.is_empty() && line != "\0" {
                if line.contains('<') || line.contains('>') {
                    // Category
                    let name = &line[1..line.len() - 1];
                    current_category = Some(String::from(name));
                    cfg.categories.push(String::from(name));
                } else if let (Some(category), Some((key, value))) =
                    (&current_category, line.split_once('\t'))
                {
                    // Key-value pair
                    cfg.settings
                        .entry(category.clone())
                        .or_insert_with(|| ConfigMap { keys: Vec::new() });
                    cfg.settings
                        .get_mut(category)?
                        .keys
                        .push((key.to_string(), value.to_string()));
                }
            }
        }

        Some(cfg)
    }

    /// Writes an existing config file to a buffer.
    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            for category in &self.categories {
                writer
                    .write_all(format!("\r\n<{}>\r\n", category).as_ref())
                    .ok()?;

                if self.settings.contains_key(category) {
                    for key in &self.settings[category].keys {
                        writer
                            .write_all(format!("{}\t{}\r\n", key.0, key.1).as_ref())
                            .ok()?;
                    }
                }
            }

            writer.write_all(b"\0").ok()?;
        }

        Some(buffer)
    }

    /// Checks if the CFG contains a key named `select_key`
    pub fn has_key(&self, select_key: &str) -> bool {
        for map in self.settings.values() {
            for (key, _) in &map.keys {
                if select_key == key {
                    return true;
                }
            }
        }

        false
    }

    /// Checks if the CFG contains a category named `select_category`
    pub fn has_category(&self, select_category: &str) -> bool {
        self.settings.contains_key(select_category)
    }

    /// Sets the value to `new_value` of `select_key`
    pub fn set_value(&mut self, select_key: &str, new_value: &str) {
        for keys in self.settings.values_mut() {
            for (key, value) in &mut keys.keys {
                if select_key == key {
                    *value = new_value.to_string();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    fn common_setup() -> ConfigFile {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("FFXIV.cfg");

        ConfigFile::from_existing(&read(d).unwrap()).unwrap()
    }

    fn common_setup_modified() -> ByteBuffer {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("FFXIV.modified.cfg");

        read(d).unwrap()
    }

    fn common_setup_invalid() -> ByteBuffer {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        read(d).unwrap()
    }

    #[test]
    fn basic_parsing() {
        let cfg = common_setup();

        assert!(cfg.has_key("TextureFilterQuality"));
        assert!(cfg.has_category("Cutscene Settings"));
    }

    #[test]
    fn basic_writing() {
        let mut cfg = common_setup();
        let modified_cfg = common_setup_modified();

        cfg.set_value("CutsceneMovieOpening", "1");

        let cfg_buffer = cfg.write_to_buffer().unwrap();

        assert_eq!(modified_cfg, cfg_buffer);
    }

    #[test]
    fn test_invalid() {
        let cfg = common_setup_invalid();

        // Feeding it invalid data should not panic
        ConfigFile::from_existing(&cfg);
    }
}
