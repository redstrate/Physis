use std::io::{BufWriter, Cursor};

use binrw::{BinRead, BinWrite, binrw, helpers::until_eof};

use crate::{ByteBuffer, ByteSpan, Platform, ReadableFile, WritableFile};

#[binrw]
#[brw(magic = b"FFXIVREPLAY\0")]
#[derive(Debug, Clone, Default)]
pub struct Replay {
    /// Only version 5 is supported right now.
    pub version: u16,
    /// The operating system this was recorded on(?)
    pub operating_system_type: u16,
    /// The client build revision this was recorded with.
    pub game_build_revision: u32,
    pub timestamp: u32,
    pub total_milliseconds: u32,
    pub displayed_milliseconds: u32,
    /// Index into the ContentFinderCondition Excel sheet. This is displayed in the Duty Recorder UI.
    #[brw(pad_after = 6)] // seems empty?
    pub content_finder_condition_id: u16,
    #[brw(pad_after = 15)] // seems empty?
    pub flags: u8,
    /// Classes that were in your party. This is displayed in the Duty Recorder UI.
    pub classjob_ids: [u8; 8],
    /// Your index into the `classjobs_ids` array.
    #[brw(pad_after = 3)] // seems empty?
    pub player_index: u8,
    pub chapters_size: u32,
    #[brw(pad_after = 28)] // seems empty?
    #[br(temp)]
    #[bw(calc = self.calculate_packets_size())]
    packets_size: u32,
    #[br(temp)]
    #[bw(calc = chapters.len() as u32)]
    chapter_count: u32,
    #[br(count = chapter_count)]
    pub chapters: Vec<ReplayChapter>,
    /// The server packets contained within this replay.
    #[br(parse_with = until_eof)]
    #[brw(pad_before = 756)] // HACK: use offset
    pub packets: Vec<ReplayPacket>,
}

#[binrw]
#[brw(repr = u8)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReplayChapterType {
    None = 0,
    Countdown = 1,
    StartRestart = 2,
    EventCutscene = 4,
    ContentStart = 5,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ReplayChapter {
    #[brw(pad_after = 3)] // unused
    pub chapter_type: ReplayChapterType,
    pub offset: u32,
    pub position_milliseconds: u32,
}

#[binrw]
#[derive(Debug, Clone)]
pub struct ReplayPacket {
    /// The IPC opcode of this packet.
    pub opcode: u16,
    #[br(temp)]
    #[bw(calc = data.len() as u16)]
    packet_data_size: u16,
    pub offset: u32,
    pub actor_id: u32,
    /// The IPC data of this packet.
    #[br(count = packet_data_size)]
    pub data: Vec<u8>,
}

impl Replay {
    fn calculate_packets_size(&self) -> u32 {
        let mut size = 0;
        for packet in &self.packets {
            size += 12 + packet.data.len() as u32;
        }
        size
    }
}

impl ReadableFile for Replay {
    fn from_existing(_platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);

        Ok(Self::read_le(&mut cursor)?)
    }
}

impl WritableFile for Replay {
    fn write_to_buffer(&self, _platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            self.write_le(&mut writer)?;
        }

        Ok(buffer)
    }
}
