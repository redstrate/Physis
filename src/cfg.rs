// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};
use crate::{ByteBuffer, ByteSpan};

/// Represents a collection of keys, mapped to their values.
#[derive(Debug)]
pub struct ConfigMap {
    /// A map of setting name to value.
    pub keys: Vec<(String, String)>,
}

/// Represents a config file, which is made up of categories and settings. Categories may have zero to one settings.
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
            settings: HashMap::new()
        };

        let cursor = Cursor::new(buffer);
        let reader = BufReader::new(cursor);

        let mut current_category: Option<String> = None;

        for (_, line) in reader.lines().enumerate() {
            let unwrap = line.unwrap();
            if !unwrap.is_empty() && unwrap != "\0" {
                if unwrap.contains('<') || unwrap.contains('>') {
                    let name = &unwrap[1..unwrap.len() - 1];
                    current_category = Some(String::from(name));
                    cfg.categories.push(String::from(name));
                } else {
                    let parts = unwrap.split_once('\t').unwrap();
                    cfg.settings.entry(current_category.clone().unwrap()).or_insert_with(|| ConfigMap{ keys: Vec::new() });

                    cfg.settings.get_mut(&current_category.clone().unwrap()).unwrap().keys.push((parts.0.to_string(), parts.1.to_string()));
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
                writer.write_all(format!("\r\n<{}>\r\n", category).as_ref()).ok()?;

                if self.settings.contains_key(category) {
                    for key in &self.settings[category].keys {
                        writer.write_all(format!("{}\t{}\r\n", key.0, key.1).as_ref()).ok()?;
                    }
                }
            }

            writer.write_all(b"\0").ok()?;
        }


        Some(buffer)
    }

    /// Checks if the CFG contains a key named `select_key`
    pub fn has_key(&self, select_key: &str) -> bool {
        for (_, keys) in &self.settings {
            for (key, _) in &keys.keys {
                if select_key == key {
                    return true;
                }
            }
        }

        false
    }

    /// Checks if the CFG contains a category named `select_category`
    pub fn has_category(&self, select_category: &str) -> bool {
        for (category, _) in &self.settings {
            if select_category == category {
                return true;
            }
        }

        false
    }

    /// Sets the value to `new_value` of `select_key`
    pub fn set_value(&mut self, select_key: &str, new_value: &str) {
        for (_, keys) in &mut self.settings {
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
}
