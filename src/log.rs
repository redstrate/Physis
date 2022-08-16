use crate::gamedata::MemoryBuffer;
use binrw::binread;
use binrw::BinRead;
use std::io::{Cursor, Seek, SeekFrom};

#[binread]
pub struct ChatLogHeader {
    content_size: u32,
    file_size: u32,

    #[br(count = file_size - content_size)]
    offset_entries: Vec<u32>,
}

#[binread]
#[br(repr = u8)]
#[derive(Debug)]
enum EventFilter {
    SystemMessages = 3,
    Unknown = 20,
    ProgressionMessage = 64,
    NPCBattle = 41,
    Unknown2 = 57,
    Unknown7 = 29,
    Unknown3 = 59,
    EnemyBattle = 170,
}

#[binread]
#[derive(Debug)]
#[br(repr = u8)]
enum EventChannel {
    System = 0,
    ServerAnnouncement = 3,
    Unknown1 = 50,
    Unknown7 = 29,
    Others = 32,
    Unknown5 = 41,
    NPCEnemy = 51,
    NPCFriendly = 59,
    Unknown4 = 64,
    Unknown6 = 170,
}

#[binread]
#[derive(Debug)]
pub struct ChatLogEntry {
    timestamp: u32,
    filter: EventFilter,
    channel: EventChannel,

    #[br(temp)]
    garbage: u32,

    #[br(ignore)]
    message: String,
}

#[derive(Debug)]
pub struct ChatLog {
    entries: Vec<ChatLogEntry>,
}

impl ChatLog {
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<ChatLog> {
        let mut cursor = Cursor::new(buffer);

        let header = ChatLogHeader::read(&mut cursor).expect("Cannot parse header.");

        let content_offset = (8 + header.file_size * 4) as u64;

        // beginning of content offset
        cursor.seek(SeekFrom::Start(content_offset)).ok()?;

        let mut entries = vec![];

        for offset in header.offset_entries {
            let new_last_offset = content_offset + offset as u64;

            let mut entry = ChatLogEntry::read(&mut cursor).expect("Unable to parse log message.");

            // TODO: handle the coloring properly, in some way
            entry.message = String::from_utf8_lossy(
                &*buffer[cursor.position() as usize..new_last_offset as usize].to_vec(),
            )
            .to_string();

            cursor.seek(SeekFrom::Start(new_last_offset)).ok()?;

            entries.push(entry);
        }

        Some(ChatLog { entries })
    }
}
