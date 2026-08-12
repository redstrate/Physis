// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use std::io::SeekFrom;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinResult;
use binrw::BinWrite;
use binrw::binrw;
use bitflags::bitflags;

/// Sound compressed data file, usually with the `.scd` file extension.
#[binrw]
#[brw(magic = b"SEDB")]
#[derive(Debug)]
pub struct Scd {
    #[bw(calc = *b"SSCF")]
    #[br(temp, assert(sub_type == *b"SSCF"))]
    sub_type: [u8; 4],

    version: u32,
    #[br(temp)]
    #[bw(calc = 0)]
    endianness: u8, // TODO: 0 = little, 1 = big
    alignment_bits: u8,
    offset: u16,
    size: u64,
    #[br(pad_after = 16)] // not read
    datetime: u64,

    #[br(temp)]
    #[bw(calc = sounds.len() as u16)]
    sound_count: u16,
    track_count: u16,
    #[br(temp)]
    #[bw(calc = audios.len() as u16)]
    audio_count: u16,
    number: u16,

    track_offset: u32,
    audio_offset: u32,
    layout_offset: u32,
    routing_offset: u32,
    attribute_offset: u32,

    #[br(pad_after = 2)] // not read
    end_of_file_padding_size: u16,

    #[br(count = sound_count)]
    #[br(restore_position)]
    sound_offsets: Vec<u32>,

    #[br(count = track_count)]
    #[br(if(track_offset != 0), seek_before = SeekFrom::Start(track_offset as u64))]
    #[br(restore_position)]
    track_offsets: Vec<u32>,

    #[br(count = audio_count)]
    #[br(if(audio_offset != 0), seek_before = SeekFrom::Start(audio_offset as u64))]
    #[br(restore_position)]
    audio_offsets: Vec<u32>,

    #[br(if(layout_offset != 0), seek_before = SeekFrom::Start(layout_offset as u64))]
    #[br(restore_position)]
    _layout_offset: u32,

    #[br(if(routing_offset != 0), seek_before = SeekFrom::Start(routing_offset as u64))]
    #[br(restore_position)]
    _routing_offset: u32,

    #[br(if(attribute_offset != 0), seek_before = SeekFrom::Start(attribute_offset as u64))]
    #[br(restore_position)]
    _attribute_offset: u32,

    #[br(restore_position, parse_with = sounds_from_offsets, args(&sound_offsets))]
    #[bw(ignore)]
    pub sounds: Vec<Sound>,

    // TODO
    // #[br(restore_position, parse_with = tracks_from_offsetsd, args(&sound_offsets))]
    // #[bw(ignore)]
    // tracks: Vec<Track>,
    #[br(restore_position, parse_with = audios_from_offsets, args(&audio_offsets))]
    #[bw(ignore)]
    pub audios: Vec<Audio>,

    #[br(if(_layout_offset != 0), seek_before = SeekFrom::Start(_layout_offset as u64))]
    pub layout: Option<SoundLayoutObject>,

    // TODO: routing
    #[br(if(_attribute_offset != 0), seek_before = SeekFrom::Start(_attribute_offset as u64))]
    pub attribute_data: Option<AttributeData>,
}

#[binrw::parser(reader, endian)]
fn sounds_from_offsets(offsets: &Vec<u32>) -> BinResult<Vec<Sound>> {
    let mut sounds: Vec<Sound> = vec![];

    for offset in offsets {
        let new_offset = *offset as u64;

        reader.seek(SeekFrom::Start(new_offset))?;
        sounds.push(Sound::read_options(reader, endian, ())?);
    }

    Ok(sounds)
}

#[binrw::parser(reader, endian)]
fn audios_from_offsets(offsets: &Vec<u32>) -> BinResult<Vec<Audio>> {
    let mut audios: Vec<Audio> = vec![];

    for offset in offsets {
        let new_offset = *offset as u64;

        reader.seek(SeekFrom::Start(new_offset))?;
        audios.push(Audio::read_options(reader, endian, ())?);
    }

    Ok(audios)
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(u8)]
pub enum SoundType {
    #[default]
    Normal = 1,
    Random,
    Stereo,
    Cycle,
    Order,
    FourChannelSurround,
    Engine,
    Dialog,

    FixedPosition = 10,
    DynamixStream,
    GroupRandom,
    GroupOrder,
    Atomosgear,
    ConditionalJump,
    Empty,

    MidiMusic = 128,
}

#[binrw]
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct SoundAttribute(i32);

bitflags! {
    impl SoundAttribute : i32 {
        const Loop = 0x0001;
        const Reverb = 0x0002;
        const FixedVolume = 0x0004;
        const FixedPosition = 0x0008;

        const Music = 0x0020;
        const BypassPLIIz = 0x0040;
        const UseExternalAttr = 0x0080;
        const ExistRoutingSetting = 0x0100;
        const MusicSurround = 0x0200;
        const BusDucking = 0x0400;
        const Acceleration = 0x0800;
        const DynamixEnd = 0x1000;
        const ExtraDesc = 0x2000;
        const DynamixPlus = 0x4000;
        const Atomosgear = 0x8000;
    }
}

impl std::fmt::Debug for SoundAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[binrw]
#[derive(Debug)]
pub struct Sound {
    #[br(temp)]
    #[bw(calc = track_infos.len() as u8)]
    track_count: u8,
    pub bus_number: u8,
    pub priority: u8,
    pub sound_type: SoundType,
    pub attribute: SoundAttribute,
    pub volume: f32,
    pub local_number: u16,
    pub user_id: u8,
    pub play_history: i8,

    #[br(if(attribute.contains(SoundAttribute::ExistRoutingSetting)))]
    pub routing_info: Option<RoutingInfo>,

    #[br(if(attribute.contains(SoundAttribute::ExtraDesc)))]
    pub extra_desc: Option<SoundExtraDesc>,

    #[br(count = track_count)]
    pub track_infos: Vec<TrackInfo>,
}

#[binrw]
#[derive(Debug)]
pub struct RoutingInfo {
    data_size: u32,
    #[brw(pad_after = 11)] // not read
    send_count: u8,
    // TODO: read data_size and send_count
}

#[binrw]
#[derive(Debug)]
pub struct SoundExtraDesc {
    #[brw(pad_after = 1)] // not read
    version: u8,
    size: u16,
    #[brw(pad_after = 8)] // not read
    play_time_length: u32,
}

#[binrw]
#[derive(Debug)]
pub struct TrackInfo {
    pub track_data_index: u16,
    pub audio_data_index: u16,
}

#[binrw]
#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub struct AudioFlag(i32);

bitflags! {
    impl AudioFlag : i32 {
        const MarkerChunk = 0x01;
        const MonoSplit = 0x02;
        const VersionShiftBit = 0x01000000;
    }
}

impl std::fmt::Debug for AudioFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        bitflags::parser::to_writer(self, f)
    }
}

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(i32)]
pub enum AudioFormat {
    #[default]
    Empty = -1,
    OggVorbis = 6, // PC music
    Mp3 = 7,       // PS3
    MsAdpcm = 12,  // PC sound
    Atrac9 = 22,   // PS4
}

#[binrw]
#[derive(Debug)]
pub struct Audio {
    size: u32,
    pub channel: u32,
    pub rate: u32,
    pub format: AudioFormat,
    pub loop_start: u32,
    pub loop_end: u32,
    sub_info_size: u32,
    pub flag: AudioFlag,

    #[br(args(&format, size, sub_info_size))]
    pub data: AudioData,
}

// TODO: Only expose the data, the header stuff isn't important
#[binrw]
#[br(import(format: &AudioFormat, size: u32, sub_info_size: u32))]
#[derive(Debug)]
pub enum AudioData {
    #[br(pre_assert(*format == AudioFormat::Empty))]
    Empty,
    #[br(pre_assert(*format == AudioFormat::OggVorbis))]
    OggVorbis {
        seek_table_header: OggVorbisSeekTableHeader,
        #[br(count = seek_table_header.seek_table_size / 4)]
        seek_table: Vec<u32>,
        #[br(parse_with = decode_ogg, args(size, &seek_table_header))]
        data: Vec<u8>,
    },
    Unknown {
        #[br(count = size)]
        data: Vec<u8>,
    },
}

#[binrw::parser(reader)]
fn decode_ogg(size: u32, header: &OggVorbisSeekTableHeader) -> BinResult<Vec<u8>> {
    let mut data: Vec<u8> = vec![0; (size + header.ogg_header_size) as usize];
    reader.read_exact(&mut data)?;

    match header.version {
        2 => {
            for data in data.iter_mut().take(header.ogg_header_size as usize) {
                *data ^= header.xor_byte;
            }
        }
        3 => {
            let byte1 = (size & 0x7F) as u8;
            let byte2 = byte1 & 0x3F;

            for i in 0..data.len() {
                let mut xor_byte = OGG_XOR_TABLE[(byte2 as usize + i) & 0xFF];
                xor_byte ^= data[i];
                xor_byte ^= byte1;
                data[i] = xor_byte;
            }
        }
        _ => {}
    }

    Ok(data)
}

/// Stolen from Lumina which stole it from FFXIVExplorer!
const OGG_XOR_TABLE: [u8; 256] = [
    0x3A, 0x32, 0x32, 0x32, 0x03, 0x7E, 0x12, 0xF7, 0xB2, 0xE2, 0xA2, 0x67, 0x32, 0x32, 0x22, 0x32,
    0x32, 0x52, 0x16, 0x1B, 0x3C, 0xA1, 0x54, 0x7B, 0x1B, 0x97, 0xA6, 0x93, 0x1A, 0x4B, 0xAA, 0xA6,
    0x7A, 0x7B, 0x1B, 0x97, 0xA6, 0xF7, 0x02, 0xBB, 0xAA, 0xA6, 0xBB, 0xF7, 0x2A, 0x51, 0xBE, 0x03,
    0xF4, 0x2A, 0x51, 0xBE, 0x03, 0xF4, 0x2A, 0x51, 0xBE, 0x12, 0x06, 0x56, 0x27, 0x32, 0x32, 0x36,
    0x32, 0xB2, 0x1A, 0x3B, 0xBC, 0x91, 0xD4, 0x7B, 0x58, 0xFC, 0x0B, 0x55, 0x2A, 0x15, 0xBC, 0x40,
    0x92, 0x0B, 0x5B, 0x7C, 0x0A, 0x95, 0x12, 0x35, 0xB8, 0x63, 0xD2, 0x0B, 0x3B, 0xF0, 0xC7, 0x14,
    0x51, 0x5C, 0x94, 0x86, 0x94, 0x59, 0x5C, 0xFC, 0x1B, 0x17, 0x3A, 0x3F, 0x6B, 0x37, 0x32, 0x32,
    0x30, 0x32, 0x72, 0x7A, 0x13, 0xB7, 0x26, 0x60, 0x7A, 0x13, 0xB7, 0x26, 0x50, 0xBA, 0x13, 0xB4,
    0x2A, 0x50, 0xBA, 0x13, 0xB5, 0x2E, 0x40, 0xFA, 0x13, 0x95, 0xAE, 0x40, 0x38, 0x18, 0x9A, 0x92,
    0xB0, 0x38, 0x00, 0xFA, 0x12, 0xB1, 0x7E, 0x00, 0xDB, 0x96, 0xA1, 0x7C, 0x08, 0xDB, 0x9A, 0x91,
    0xBC, 0x08, 0xD8, 0x1A, 0x86, 0xE2, 0x70, 0x39, 0x1F, 0x86, 0xE0, 0x78, 0x7E, 0x03, 0xE7, 0x64,
    0x51, 0x9C, 0x8F, 0x34, 0x6F, 0x4E, 0x41, 0xFC, 0x0B, 0xD5, 0xAE, 0x41, 0xFC, 0x0B, 0xD5, 0xAE,
    0x41, 0xFC, 0x3B, 0x70, 0x71, 0x64, 0x33, 0x32, 0x12, 0x32, 0x32, 0x36, 0x70, 0x34, 0x2B, 0x56,
    0x22, 0x70, 0x3A, 0x13, 0xB7, 0x26, 0x60, 0xBA, 0x1B, 0x94, 0xAA, 0x40, 0x38, 0x00, 0xFA, 0xB2,
    0xE2, 0xA2, 0x67, 0x32, 0x32, 0x12, 0x32, 0xB2, 0x32, 0x32, 0x32, 0x32, 0x75, 0xA3, 0x26, 0x7B,
    0x83, 0x26, 0xF9, 0x83, 0x2E, 0xFF, 0xE3, 0x16, 0x7D, 0xC0, 0x1E, 0x63, 0x21, 0x07, 0xE3, 0x01,
];

#[binrw]
#[derive(Debug)]
pub struct OggVorbisSeekTableHeader {
    version: u8,
    struct_size: u8,
    xor_byte: u8,
    unk: [u8; 9],
    step: f32,
    seek_table_size: u32,
    #[brw(pad_after = 8)] // not read
    ogg_header_size: u32,
}

#[binrw]
#[brw(repr = u8)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
#[repr(u8)]
pub enum SoundLayoutObjectType {
    #[default]
    Null,
    Ambient,
    Direction,
    Point,
    PointDir,
    Line,
    Polyline,
    Surface,
    BoardObstruction,
    BoxObstruction,
    PolylineObstruction,
    Polygon,
    BoxExtController,
    LineExtController,
    PolygonObstruction,
}

#[binrw]
#[derive(Debug)]
pub struct SoundLayoutObject {
    size: u16,
    pub object_type: SoundLayoutObjectType,
    pub version: u8,
    pub flags: u8,
    pub group_number: u8,
    pub local_id: u16,
    pub bank_id: u32,
    pub flag2: u8,
    pub reverb_type: u8,
    pub ab_group_number: u16,
    pub array_volume: [f32; 4],
}

#[binrw]
#[derive(Debug)]
pub struct AttributeData {
    #[brw(pad_after = 1)] // not read
    version: u8,
    pub attribute_id: u16,
    pub search_attribute_id: u16,
    pub condition: u8,
    arg_count: u8,
    sound_label_low: u32,
    sound_label_high: u32,
}

impl ReadableFile for Scd {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(Self::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

impl WritableFile for Scd {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Scd>();
    }
}
