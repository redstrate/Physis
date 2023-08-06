// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::io::{BufRead, BufReader, BufWriter, Cursor, Write};

use crate::cfg;
use crate::gamedata::MemoryBuffer;

#[derive(Debug)]
pub struct CFGSetting {
    pub keys: HashMap<String, String>,
}

#[derive(Debug)]
pub struct CFG {
    pub categories: Vec<String>,
    pub settings: HashMap<String, CFGSetting>,
}

impl CFG {
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<CFG> {
        let mut cfg = CFG {
            categories: Vec::new(),
            settings: HashMap::new()
        };

        let cursor = Cursor::new(buffer);
        let reader = BufReader::new(cursor);

        let mut current_category: Option<String> = None;

        for (_, line) in reader.lines().enumerate() {
            // now parse the line!

            let unwrap = line.unwrap();
            println!("{}", unwrap);
            if !unwrap.is_empty() {
                if unwrap.contains('<') || unwrap.contains('>') {
                    let name = &unwrap[1..unwrap.len() - 1];
                    println!("{}", name);
                    current_category = Some(String::from(name));
                    cfg.categories.push(String::from(name));
                } else {
                    let parts = unwrap.split_once('\t').unwrap();
                    if !cfg.settings.contains_key(&current_category.clone().unwrap()) {
                        cfg.settings.insert(current_category.clone().unwrap(), cfg::CFGSetting{ keys: HashMap::new() });
                    }

                    cfg.settings.get_mut(&current_category.clone().unwrap()).unwrap().keys.insert(parts.0.to_string(), parts.1.to_string());
                }
            }
        }

        Some(cfg)
    }

    pub fn write_to_buffer(&self) -> Option<MemoryBuffer> {
        let mut buffer = MemoryBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            for category in &self.categories {
                writer.write(format!("\n<{}>", category).as_ref());

                if self.settings.contains_key(category) {
                    for key in &self.settings[category].keys {
                        writer.write(format!("\n{}\t{}", key.0, key.1).as_ref());
                    }
                }

                writer.write(b"\n");
            }
        }

        Some(buffer)
    }
}
