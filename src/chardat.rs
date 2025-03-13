// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufWriter, Cursor};

use crate::common_file_operations::{read_bool_from, read_string, write_bool_as, write_string};
use crate::{ByteBuffer, ByteSpan};
use binrw::binrw;
use binrw::{BinRead, BinWrite};

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
        _ => Race::Hyur,
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
        Race::Viera => 8,
    }
}

fn convert_dat_gender(x: u8) -> Gender {
    match x {
        0 => Gender::Male,
        1 => Gender::Female,
        _ => Gender::Male,
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
        9 => Subrace::SeaWolf,
        10 => Subrace::Hellsguard,
        11 => Subrace::Raen,
        12 => Subrace::Xaela,
        13 => Subrace::Hellion,
        14 => Subrace::Lost,
        15 => Subrace::Rava,
        16 => Subrace::Veena,
        _ => Subrace::Midlander,
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
        Subrace::SeaWolf => 9,
        Subrace::Hellsguard => 10,
        Subrace::Raen => 11,
        Subrace::Xaela => 12,
        Subrace::Hellion => 13,
        Subrace::Lost => 14,
        Subrace::Rava => 15,
        Subrace::Veena => 16,
    }
}

#[binrw]
#[br(little)]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct CustomizeData {
    /// The race of the character.
    #[br(map = convert_dat_race)]
    #[bw(map = convert_race_dat)]
    pub race: Race,

    /// The gender of the character.
    #[br(map = convert_dat_gender)]
    #[bw(map = convert_gender_dat)]
    pub gender: Gender,

    /// The age of the character. Normal = 1, Old = 3, Young = 4.
    pub age: u8,

    /// The height of the character from 0 to 100.
    pub height: u8,

    /// The character's subrace.
    #[br(map = convert_dat_subrace)]
    #[bw(map = convert_subrace_dat)]
    pub subrace: Subrace,

    /// The character's selected face.
    pub face: u8,

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

    /// The color of the character's facial features.'
    pub facial_feature_color: u8,

    /// The character's selected eyebrows.
    pub eyebrows: u8,

    /// The character's left eye color.
    pub left_eye_color: u8,

    /// The character's selected eyes.
    /// If the character has selected "Small Iris" then it adds 128 to this.
    pub eyes: u8,

    /// The character's selected nose.
    pub nose: u8,

    /// The character's selected jaw.
    pub jaw: u8,

    /// The character's selected mouth.
    pub mouth: u8,

    /// The character's selected pattern.
    pub lips_tone_fur_pattern: u8,

    /// Depending on the race, it's either the ear size, muscle size, or tail size.
    pub race_feature_size: u8,

    /// Depending on the race, it's the selected tail or ears.
    pub race_feature_type: u8,

    /// The size of the character's bust from 0 to 100.
    pub bust: u8,

    /// The character's choice of face paint.
    pub face_paint: u8,

    /// The color of the face paint.
    /// If the character has selected "Light" then it adds 128 to this.
    pub face_paint_color: u8,

    /// The character's chosen voice.
    pub voice: u8,
}

const MAX_COMMENT_LENGTH: usize = 164;

/// Represents the several options that make up a character data file (DAT) which is used by the game's character creation system to save and load presets.
#[binrw]
#[br(little)]
#[repr(C)]
#[brw(magic = 0x2013FF14u32)]
#[derive(Debug)]
pub struct CharacterData {
    /// The "version" of the game this was created with.
    /// Always corresponds to the released expansion at the time, e.g. A Realm Reborn character will have a version of 1. A Shadowbringers character will have a version of 4.
    pub version: u32,

    /// The checksum of the data fields.
    // TODO: should we re-expose this checksum?
    #[brw(pad_after = 4)]
    #[bw(calc = self.calc_checksum())]
    pub checksum: u32,

    pub customize: CustomizeData,

    /// The timestamp when the preset was created.
    /// This is a UTC time in seconds since the Unix epoch.
    #[brw(pad_before = 1)]
    pub timestamp: u32,

    // TODO: this is terrible, just read until string nul terminator
    #[br(count = MAX_COMMENT_LENGTH)]
    #[bw(pad_size_to = MAX_COMMENT_LENGTH)]
    #[br(map = read_string)]
    #[bw(map = write_string)]
    pub comment: String,
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

    fn calc_checksum(&self) -> u32 {
        let mut buffer = ByteBuffer::new();

        {
            let cursor = Cursor::new(&mut buffer);
            let mut writer = BufWriter::new(cursor);

            self.customize.write_le(&mut writer).unwrap();
        }

        // The checksum also considers the timestamp and the comment
        buffer.push(0x00);
        buffer.extend_from_slice(&self.timestamp.to_le_bytes());

        let mut comment = write_string(&self.comment);
        comment.resize(MAX_COMMENT_LENGTH, 0);
        buffer.extend_from_slice(&comment);

        let mut checksum: u32 = 0;
        for (i, byte) in buffer.iter().enumerate() {
            checksum ^= (*byte as u32) << (i % 24);
        }

        checksum
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

    fn common_setup(name: &str) -> CharacterData {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests/chardat");
        d.push(name);

        CharacterData::from_existing(&read(d).unwrap()).unwrap()
    }

    #[test]
    fn read_arr() {
        let chardat = common_setup("arr.dat");

        assert_eq!(chardat.version, 1);
        assert_eq!(chardat.customize.race, Race::Hyur);
        assert_eq!(chardat.customize.gender, Gender::Male);
        assert_eq!(chardat.customize.age, 1);
        assert_eq!(chardat.customize.height, 50);
        assert_eq!(chardat.customize.subrace, Subrace::Midlander);
        assert_eq!(chardat.customize.face, 5);
        assert_eq!(chardat.customize.hair, 1);
        assert!(!chardat.customize.enable_highlights);
        assert_eq!(chardat.customize.skin_tone, 2);
        assert_eq!(chardat.customize.right_eye_color, 37);
        assert_eq!(chardat.customize.hair_tone, 53);
        assert_eq!(chardat.customize.highlights, 0);
        assert_eq!(chardat.customize.facial_features, 2);
        assert_eq!(chardat.customize.facial_feature_color, 2);
        assert_eq!(chardat.customize.eyebrows, 0);
        assert_eq!(chardat.customize.left_eye_color, 37);
        assert_eq!(chardat.customize.eyes, 0);
        assert_eq!(chardat.customize.nose, 0);
        assert_eq!(chardat.customize.jaw, 0);
        assert_eq!(chardat.customize.mouth, 0);
        assert_eq!(chardat.customize.lips_tone_fur_pattern, 43);
        assert_eq!(chardat.customize.race_feature_size, 50);
        assert_eq!(chardat.customize.race_feature_type, 0);
        assert_eq!(chardat.customize.bust, 0);
        assert_eq!(chardat.customize.face_paint_color, 36);
        assert_eq!(chardat.customize.face_paint, 0);
        assert_eq!(chardat.customize.voice, 1);
        assert_eq!(chardat.comment, "Custom Comment Text");
    }

    #[test]
    fn read_heavensward() {
        let chardat = common_setup("heavensward.dat");

        assert_eq!(chardat.version, 2);
        assert_eq!(chardat.customize.race, Race::AuRa);
        assert_eq!(chardat.customize.gender, Gender::Female);
        assert_eq!(chardat.customize.age, 1);
        assert_eq!(chardat.customize.height, 50);
        assert_eq!(chardat.customize.subrace, Subrace::Xaela);
        assert_eq!(chardat.customize.face, 3);
        assert_eq!(chardat.customize.hair, 5);
        assert!(!chardat.customize.enable_highlights);
        assert_eq!(chardat.customize.skin_tone, 160);
        assert_eq!(chardat.customize.right_eye_color, 91);
        assert_eq!(chardat.customize.hair_tone, 159);
        assert_eq!(chardat.customize.highlights, 0);
        assert_eq!(chardat.customize.facial_features, 127);
        assert_eq!(chardat.customize.facial_feature_color, 99);
        assert_eq!(chardat.customize.eyebrows, 0);
        assert_eq!(chardat.customize.left_eye_color, 91);
        assert_eq!(chardat.customize.eyes, 0);
        assert_eq!(chardat.customize.nose, 0);
        assert_eq!(chardat.customize.jaw, 0);
        assert_eq!(chardat.customize.mouth, 0);
        assert_eq!(chardat.customize.lips_tone_fur_pattern, 0);
        assert_eq!(chardat.customize.race_feature_size, 50);
        assert_eq!(chardat.customize.race_feature_type, 1);
        assert_eq!(chardat.customize.bust, 25);
        assert_eq!(chardat.customize.face_paint_color, 0);
        assert_eq!(chardat.customize.face_paint, 0);
        assert_eq!(chardat.customize.voice, 112);
        assert_eq!(chardat.comment, "Heavensward Comment Text");
    }

    #[test]
    fn read_stormblood() {
        let chardat = common_setup("stormblood.dat");

        assert_eq!(chardat.version, 3);
        assert_eq!(chardat.customize.race, Race::Lalafell);
        assert_eq!(chardat.customize.gender, Gender::Male);
        assert_eq!(chardat.customize.age, 1);
        assert_eq!(chardat.customize.height, 50);
        assert_eq!(chardat.customize.subrace, Subrace::Plainsfolk);
        assert_eq!(chardat.customize.face, 1);
        assert_eq!(chardat.customize.hair, 8);
        assert!(!chardat.customize.enable_highlights);
        assert_eq!(chardat.customize.skin_tone, 25);
        assert_eq!(chardat.customize.right_eye_color, 11);
        assert_eq!(chardat.customize.hair_tone, 45);
        assert_eq!(chardat.customize.highlights, 0);
        assert_eq!(chardat.customize.facial_features, 0);
        assert_eq!(chardat.customize.facial_feature_color, 2);
        assert_eq!(chardat.customize.eyebrows, 0);
        assert_eq!(chardat.customize.left_eye_color, 11);
        assert_eq!(chardat.customize.eyes, 0);
        assert_eq!(chardat.customize.nose, 0);
        assert_eq!(chardat.customize.jaw, 0);
        assert_eq!(chardat.customize.mouth, 0);
        assert_eq!(chardat.customize.lips_tone_fur_pattern, 43);
        assert_eq!(chardat.customize.race_feature_size, 25);
        assert_eq!(chardat.customize.race_feature_type, 2);
        assert_eq!(chardat.customize.bust, 0);
        assert_eq!(chardat.customize.face_paint_color, 36);
        assert_eq!(chardat.customize.face_paint, 0);
        assert_eq!(chardat.customize.voice, 19);
        assert_eq!(chardat.comment, "Stormblood Comment Text");
    }

    #[test]
    fn read_shadowbringers() {
        let chardat = common_setup("shadowbringers.dat");

        assert_eq!(chardat.version, 4);
        assert_eq!(chardat.customize.race, Race::Viera);
        assert_eq!(chardat.customize.gender, Gender::Female);
        assert_eq!(chardat.customize.age, 1);
        assert_eq!(chardat.customize.height, 50);
        assert_eq!(chardat.customize.subrace, Subrace::Rava);
        assert_eq!(chardat.customize.face, 1);
        assert_eq!(chardat.customize.hair, 8);
        assert!(!chardat.customize.enable_highlights);
        assert_eq!(chardat.customize.skin_tone, 12);
        assert_eq!(chardat.customize.right_eye_color, 43);
        assert_eq!(chardat.customize.hair_tone, 53);
        assert_eq!(chardat.customize.highlights, 0);
        assert_eq!(chardat.customize.facial_features, 4);
        assert_eq!(chardat.customize.facial_feature_color, 0);
        assert_eq!(chardat.customize.eyebrows, 2);
        assert_eq!(chardat.customize.left_eye_color, 43);
        assert_eq!(chardat.customize.eyes, 131);
        assert_eq!(chardat.customize.nose, 2);
        assert_eq!(chardat.customize.jaw, 1);
        assert_eq!(chardat.customize.mouth, 131);
        assert_eq!(chardat.customize.lips_tone_fur_pattern, 171);
        assert_eq!(chardat.customize.race_feature_size, 50);
        assert_eq!(chardat.customize.race_feature_type, 2);
        assert_eq!(chardat.customize.bust, 100);
        assert_eq!(chardat.customize.face_paint_color, 131);
        assert_eq!(chardat.customize.face_paint, 3);
        assert_eq!(chardat.customize.voice, 160);
        assert_eq!(chardat.comment, "Shadowbringers Comment Text");
    }

    #[test]
    fn write_shadowbringers() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests/chardat");
        d.push("shadowbringers.dat");

        let chardat_bytes = &read(d).unwrap();
        let chardat = CharacterData::from_existing(chardat_bytes).unwrap();
        assert_eq!(*chardat_bytes, chardat.write_to_buffer().unwrap());
    }
}
