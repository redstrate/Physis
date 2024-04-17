// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufWriter, Cursor};

use binrw::{BinRead, BinWrite};
use binrw::binrw;
use crate::{ByteBuffer, ByteSpan};
use crate::cfg::ConfigFile;
use crate::common_file_operations::{read_bool_from, write_bool_as};

use crate::race::{Gender, Race, Subrace};

fn convert_dat_race(x: u8) -> Race {
    match x {
        1 => Race::Hyur,
        2 => Race::Elezen,
        3 => Race::Lalafell,
        4 => Race::Miqote,
        5 => Race::Roegadyn,
        6 => Race::AuRa,
        7 => Race::Hrothgar,
        8 => Race::Viera,
        _ => Race::Hyur
    }
}

fn convert_race_dat(race: &Race) -> u8 {
    match race {
        Race::Hyur => 1,
        Race::Elezen => 2,
        Race::Lalafell => 3,
        Race::Miqote => 4,
        Race::Roegadyn => 5,
        Race::AuRa => 6,
        Race::Hrothgar => 7,
        Race::Viera => 8
    }
}

fn convert_dat_gender(x: u8) -> Gender {
    match x {
        0 => Gender::Male,
        1 => Gender::Female,
        _ => Gender::Male
    }
}

fn convert_gender_dat(gender: &Gender) -> u8 {
    match gender {
        Gender::Male => 0,
        Gender::Female => 1,
    }
}

fn convert_dat_subrace(x: u8) -> Subrace {
    match x {
        1 => Subrace::Midlander,
        2 => Subrace::Highlander,
        3 => Subrace::Wildwood,
        4 => Subrace::Duskwight,
        5 => Subrace::Plainsfolk,
        6 => Subrace::Dunesfolk,
        7 => Subrace::Seeker,
        8 => Subrace::Keeper,
        9 => Subrace:: SeaWolf,
        10 => Subrace::Hellsguard,
        11 => Subrace::Raen,
        12 => Subrace::Xaela,
        13 => Subrace::Hellion,
        14 => Subrace::Lost,
        15 => Subrace::Rava,
        16 => Subrace::Veena,
        _ => Subrace::Midlander
    }
}

fn convert_subrace_dat(subrace: &Subrace) -> u8 {
    match subrace {
        Subrace::Midlander => 1,
        Subrace::Highlander => 2,
        Subrace::Wildwood => 3,
        Subrace::Duskwight => 4,
        Subrace::Plainsfolk => 5,
        Subrace::Dunesfolk => 6,
        Subrace::Seeker => 7,
        Subrace::Keeper => 8,
        Subrace:: SeaWolf => 9,
        Subrace::Hellsguard => 10,
        Subrace::Raen => 11,
        Subrace::Xaela => 12,
        Subrace::Hellion => 13,
        Subrace::Lost => 14,
        Subrace::Rava => 15,
        Subrace::Veena => 16
    }
}

/// Represents the several options that make up a character data file (DAT) which is used by the game's character creation system to save and load presets.
#[binrw]
#[br(little)]
#[repr(C)]
#[br(magic = 0x2013FF14u32)]
#[derive(Debug)]
pub struct CharacterData { // version 4
    /// The version of the character data, the only supported version right now is 4.
    pub version: u32,

    /// The checksum of the data fields.
    #[br(pad_after = 4)]
    pub checksum: u32,

    /// The race of the character.
    #[br(map = convert_dat_race )]
    #[bw(map = convert_race_dat )]
    pub race: Race,

    /// The gender of the character.
    #[br(map = convert_dat_gender )]
    #[bw(map = convert_gender_dat )]
    pub gender: Gender,

    /// The age of the character. Normal = 1, Old = 3, Young = 4.
    pub age: u8,

    /// The height of the character.
    pub height: u8,

    /// The character's subrace.
    #[br(map = convert_dat_subrace )]
    #[bw(map = convert_subrace_dat )]
    pub subrace: Subrace,

    /// The character's selected head.
    pub head: u8,

    /// The character's selected hair.
    pub hair: u8,

    /// If hair highlights are enabled for this character.
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub enable_highlights: bool,

    /// The character's skin tone.
    pub skin_tone: u8,

    /// The character's right eye color.
    pub right_eye_color: u8,

    /// The character's hair color.
    pub hair_tone: u8,

    /// The color of the hair highlights.
    pub highlights: u8,

    /// The selected facial features.
    pub facial_features: u8,

    /// If the character has limbal eyes.
    pub limbal_eyes: u8,

    /// The character's selected eyebrows.
    pub eyebrows: u8,

    /// The character's left eye color.
    pub left_eye_color: u8,

    /// The character's selected eyes.
    pub eyes: u8,

    /// The character's selected nose.
    pub nose: u8,

    /// The character's selected jaw.
    pub jaw: u8,

    /// The character's selected mouth.
    pub mouth: u8,

    /// The character's selected pattern.
    pub lips_tone_fur_pattern: u8,

    /// The character's selected tail.
    pub tail: u8,

    /// The character's choice of face paint.
    pub face_paint: u8,

    /// The size of the character's bust.
    pub bust: u8,

    /// The color of the face paint.
    pub face_paint_color: u8,

    /// The character's chosen voice.
    pub voice: u8,

    /// The timestamp when the preset was created.
    #[br(pad_before = 1)]
    pub timestamp: [u8; 4]
}

impl CharacterData {
    /// Parses existing character data.
    pub fn from_existing(buffer: ByteSpan) -> Option<CharacterData> {
        let mut cursor = Cursor::new(buffer);

        CharacterData::read(&mut cursor).ok()
    }

    /// Write existing character data to a buffer.
    pub fn write_to_buffer(&self) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            self.write_le(&mut writer).ok()?;
        }

        Some(buffer)
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
        CharacterData::from_existing(&read(d).unwrap());
    }
}
