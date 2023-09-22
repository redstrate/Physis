// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};

use crate::cfg;
use crate::gamedata::MemoryBuffer;

/// Represents a collection of keys, mapped to their values.
#[derive(Debug)]
pub struct ConfigMap {
    /// A map of setting name to value.
    pub keys: HashMap<String, String>,
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
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<ConfigFile> {
        let mut cfg = ConfigFile {
            categories: Vec::new(),
            settings: HashMap::new()
        };

        let cursor = Cursor::new(buffer);
        let reader = BufReader::new(cursor);

        let mut current_category: Option<String> = None;

        for (_, line) in reader.lines().enumerate() {
            // now parse the line!

            let unwrap = line.unwrap();
            if !unwrap.is_empty() {
                if unwrap.contains('<') || unwrap.contains('>') {
                    let name = &unwrap[1..unwrap.len() - 1];
                    println!("{}", name);
                    current_category = Some(String::from(name));
                    cfg.categories.push(String::from(name));
                } else {
                    let parts = unwrap.split_once('\t').unwrap();
                    cfg.settings.entry(current_category.clone().unwrap()).or_insert_with(|| cfg::ConfigMap{ keys: HashMap::new() });

                    cfg.settings.get_mut(&current_category.clone().unwrap()).unwrap().keys.insert(parts.0.to_string(), parts.1.to_string());
                }
            }
        }

        Some(cfg)
    }

    /// Writes an existing config file to a buffer.
    pub fn write_to_buffer(&self) -> Option<MemoryBuffer> {
        let mut buffer = MemoryBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            for category in &self.categories {
                writer.write_all(format!("\n<{}>", category).as_ref()).ok()?;

                if self.settings.contains_key(category) {
                    for key in &self.settings[category].keys {
                        writer.write_all(format!("\n{}\t{}", key.0, key.1).as_ref()).ok()?;
                    }
                }

                writer.write_all(b"\n").ok()?;
            }
        }

        Some(buffer)
    }
}
