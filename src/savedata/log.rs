// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use crate::ByteSpan;
use binrw::BinRead;
use binrw::binrw;

#[binrw]
#[allow(dead_code)]
#[brw(little)]
struct ChatLogHeader {
    content_size: u32,
    file_size: u32,

    #[br(count = file_size.saturating_sub(content_size))]
    offset_entries: Vec<u32>,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug)]
pub enum EventFilter {
    SystemMessages = 3,
    Unknown = 20,
    ProgressionMessage = 64,
    NPCBattle = 41,
    Unknown2 = 57,
    Unknown7 = 29,
    Unknown3 = 59,
    EnemyBattle = 170,
    Unknown4 = 581,
    Unknown8 = 43,
    Unknown9 = 60,
    Unknown10 = 61,
    Unknown11 = 62,
    Unknown12 = 58,
    Unknown13 = 169,
    Unknown14 = 175,
    Unknown15 = 42,
    Unknown16 = 171,
    Unknown17 = 177,
    Unknown18 = 174,
    Unknown19 = 47,
    Unknown20 = 176,
    Unknown21 = 44,
    Unknown22 = 173,
    Unknown23 = 46,
    Unknown24 = 10,
    Unknown25 = 185,
    Unknown26 = 190,
    Unknown27 = 11,
    Unknown28 = 70,
    Unknown29 = 105,
}

#[binrw]
#[derive(Debug)]
#[brw(repr = u8)]
pub enum EventChannel {
    System = 0,
    Unknown8 = 2,
    ServerAnnouncement = 3,
    Unknown9 = 8,
    Unknown1 = 50,
    Unknown7 = 29,
    Others = 32,
    Unknown5 = 41,
    NPCEnemy = 51,
    NPCFriendly = 59,
    Unknown4 = 64,
    Unknown6 = 170,
    Unknown10 = 10,
    Unknown11 = 66,
    Unknown12 = 44,
    Unknown13 = 40,
    Unknown14 = 42,
    Unknown15 = 11,
    Unknown16 = 67,
    Unknown17 = 68,
    Unknown18 = 34,
    Unknown19 = 110,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
#[brw(little)]
/// Represents an entry in the chat log
pub struct ChatLogEntry {
    timestamp: u32,
    /// The event filter
    pub filter: EventFilter,
    /// The event channel
    pub channel: EventChannel,

    #[br(temp)]
    #[bw(calc = 1)]
    _garbage: u32,

    /// The message
    #[brw(ignore)]
    pub message: String,
}

#[derive(Debug)]
#[allow(dead_code)]
/// Chat log, which contains previously recorded messages from other players
pub struct ChatLog {
    pub entries: Vec<ChatLogEntry>,
}

impl ChatLog {
    /// Read an existing file.
    pub fn from_existing(buffer: ByteSpan) -> Option<ChatLog> {
        let mut cursor = Cursor::new(buffer);

        let header = ChatLogHeader::read(&mut cursor).expect("Cannot parse header.");
        // Dumb check for obviously wrong values
        if header.content_size as usize > buffer.len() || header.file_size as usize > buffer.len() {
            return None;
        }

        let content_offset = (8 + header.file_size * 4) as u64;

        // beginning of content offset
        //cursor.seek(SeekFrom::Start(content_offset)).ok()?;

        let mut entries = vec![];

        for (i, offset) in header.offset_entries.iter().enumerate() {
            let new_last_offset = content_offset + *offset as u64;

            cursor.seek(SeekFrom::Start(new_last_offset)).ok()?;

            let mut entry = ChatLogEntry::read(&mut cursor).expect("Unable to parse log message.");

            let next_offset = if i + 1 == header.offset_entries.len() {
                buffer.len()
            } else {
                (content_offset + header.offset_entries[i + 1] as u64) as usize
            };

            // TODO: handle the coloring properly, in some way
            entry.message =
                String::from_utf8_lossy(&buffer[cursor.position() as usize..next_offset])
                    .to_string();

            entries.push(entry);
        }

        Some(ChatLog { entries })
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
        ChatLog::from_existing(&read(d).unwrap());
    }
}
