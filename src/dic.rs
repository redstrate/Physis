// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom};

use crate::ByteSpan;
use binrw::binrw;
use binrw::{BinRead, BinReaderExt};

// Based off of https://github.com/Lotlab/ffxiv-vulgar-words-reader/
// Credit goes to Jim Kirisame for documenting this format
// TODO: double check I'm reading everything correctly

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct EntryItem {
    flag: u32,
    sibling: u32,
    child: u32,
    offset: u32,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct DictionaryHeader {
    #[br(seek_before = SeekFrom::Start(0x8124))]
    #[br(count = 256)]
    chara_replace1: Vec<u16>,

    #[br(count = 256)]
    chara_replace2: Vec<u16>,

    #[br(count = 256)]
    chara_replace3: Vec<u16>,

    #[br(count = 5)]
    block_offsets: Vec<u32>,

    #[br(count = 5)]
    block_lengths: Vec<u32>,

    #[br(pad_before = 4)]
    #[br(count = 256)]
    chara_block: Vec<u32>,

    #[br(ignore)]
    begin_node: Vec<u16>,

    #[br(ignore)]
    inner_node: Vec<u16>,

    #[br(ignore)]
    chara: Vec<u16>,

    #[br(ignore)]
    word: Vec<u16>,

    #[br(ignore)]
    entries: Vec<EntryItem>,
}

pub struct Dictionary {
    header: DictionaryHeader,
    pub words: Vec<String>,
}

impl Dictionary {
    /// Parses an existing dictionary file.
    pub fn from_existing(buffer: ByteSpan) -> Option<Dictionary> {
        let mut cursor = Cursor::new(buffer);
        let mut dict = DictionaryHeader::read(&mut cursor).unwrap();

        let map_start = 0x8750u32;
        let map_size = 0x200u32;

        // fix up offsets
        for offset in &mut dict.block_offsets {
            *offset = *offset + map_start + map_size;
        }

        for i in 0..dict.block_lengths[0] / 2 {
            let offset = dict.block_offsets[0] + i * 2;
            cursor.seek(SeekFrom::Start(offset as u64)).ok()?;
            dict.begin_node.push(cursor.read_le::<u16>().ok()?);
        }

        for i in 0..dict.block_lengths[1] / 2 {
            let offset = dict.block_offsets[1] + i * 2;
            cursor.seek(SeekFrom::Start(offset as u64)).ok()?;
            dict.inner_node.push(cursor.read_le::<u16>().ok()?);
        }

        for i in 0..dict.block_lengths[2] / 2 {
            let offset = dict.block_offsets[2] + i * 2;
            cursor.seek(SeekFrom::Start(offset as u64)).ok()?;
            dict.chara.push(cursor.read_le::<u16>().ok()?);
        }

        for i in 0..dict.block_lengths[3] / 2 {
            let offset = dict.block_offsets[3] + i * 2;
            cursor.seek(SeekFrom::Start(offset as u64)).ok()?;
            dict.word.push(cursor.read_le::<u16>().ok()?);
        }

        for i in 0..dict.block_lengths[4] / 16 {
            let offset = dict.block_offsets[4] + i * 16;
            cursor.seek(SeekFrom::Start(offset as u64)).ok()?;
            dict.entries.push(cursor.read_le::<EntryItem>().ok()?);
        }

        let mut dict = Dictionary {
            header: dict,
            words: Vec::new(),
        };

        // TODO: lol
        dict.words = dict.list_words()?;

        Some(dict)
    }

    fn list_words(&self) -> Option<Vec<String>> {
        let mut result = Vec::new();
        let lut = self.generate_index_rune_lookup_table();
        for (id, v) in self.header.begin_node.iter().enumerate() {
            if *v == 0 {
                continue;
            }

            let chara = Dictionary::index_to_rune(&lut, id as u32);
            self.dump_dict_node(&mut result, *v as i32, String::from(chara as u8 as char))
        }

        Some(result)
    }

    fn generate_index_rune_lookup_table(&self) -> HashMap<u16, u16> {
        let mut map = HashMap::new();
        for i in 0..self.header.chara_block.len() {
            map.insert(self.header.chara_block[i] as u16, i as u16);
        }

        map
    }

    fn index_to_rune(lookup_table: &HashMap<u16, u16>, index: u32) -> i32 {
        let higher = index >> 8;
        let lower = index & 0xFF;

        if higher == 0 {
            return 0;
        }

        if let Some(new_val) = lookup_table.get(&(higher as u16)) {
            (((*new_val as u32) << 8) + lower) as i32
        } else {
            0
        }
    }

    fn dump_dict_node(&self, vec: &mut Vec<String>, entry_id: i32, prev: String) {
        let node = &self.header.entries[entry_id as usize];
        for i in 0..node.sibling {
            let Some(current) = self.get_string(entry_id, i as i32) else {
                return;
            };

            if node.child == 0 {
                vec.push(prev.clone() + &current);
                continue;
            }

            let value = self.header.inner_node[(node.child + i) as usize];
            if value == 0 {
                vec.push(prev.clone() + &current);
                continue;
            }

            self.dump_dict_node(vec, value as i32, prev.clone() + &current);
        }
    }

    fn get_string(&self, entry_id: i32, sibling_id: i32) -> Option<String> {
        if let Some(characters) = self.get_string_characters(entry_id, sibling_id) {
            return String::from_utf16(&characters).ok();
        }

        None
    }

    fn get_string_characters(&self, entry_id: i32, sibling_id: i32) -> Option<Vec<u16>> {
        if entry_id as usize >= self.header.entries.len() {
            return None;
        }

        let entry = self.header.entries.get(entry_id as usize)?;

        if entry.flag == 0 {
            let pos = (entry.offset / 2) as i32 + sibling_id;
            if pos as usize > self.header.chara.len() {
                return None;
            }

            if self.header.chara[pos as usize] == 0 {
                return None;
            }

            return Some(vec![self.header.chara[pos as usize]]);
        }

        let begin = entry.offset / 2;
        let mut end = begin + 1;

        while (end as usize) < self.header.word.len() && self.header.word[end as usize] != 0 {
            end += 1;
        }

        Some(self.header.word[begin as usize..end as usize].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Dictionary::from_existing(&read(d).unwrap());
    }
}
