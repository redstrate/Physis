// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{BufWriter, Cursor};

use crate::common_file_operations::{read_bool_from, read_string, write_bool_as, write_string};
use crate::{ByteBuffer, ByteSpan};
use binrw::binrw;
use binrw::{BinRead, BinWrite};

use crate::race::{Gender, Race, Tribe};

#[binrw]
#[br(little)]
#[repr(C)]
#[derive(Clone, Debug)]
pub struct CustomizeData {
    /// The race of the character.
    pub race: Race,

    /// The gender of the character.
    pub gender: Gender,

    /// The age of the character. Normal = 1, Old = 3, Young = 4.
    pub age: u8,

    /// The height of the character from 0 to 100.
    pub height: u8,

    /// The character's tribe.
    pub tribe: Tribe,

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

impl Default for CustomizeData {
    fn default() -> Self {
        Self {
            race: Race::Hyur,
            tribe: Tribe::Midlander,
            gender: Gender::Male,
            age: 1,
            height: 50,
            face: 1,
            hair: 1,
            enable_highlights: false,
            skin_tone: 1,
            right_eye_color: 1,
            hair_tone: 1,
            highlights: 1,
            facial_features: 0,
            facial_feature_color: 1,
            eyebrows: 1,
            left_eye_color: 1,
            eyes: 1,
            nose: 1,
            jaw: 1,
            mouth: 1,
            lips_tone_fur_pattern: 0,
            race_feature_size: 0,
            race_feature_type: 0,
            bust: 0,
            face_paint: 0,
            face_paint_color: 1,
            voice: 1,
        }
    }
}

const MAX_COMMENT_LENGTH: usize = 164;

/// Represents the several options that make up a character data file (DAT) which is used by the game's character creation system to save and load presets.
#[binrw]
#[br(little)]
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
        assert_eq!(chardat.customize.tribe, Tribe::Midlander);
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
        assert_eq!(chardat.customize.tribe, Tribe::Xaela);
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
        assert_eq!(chardat.customize.tribe, Tribe::Plainsfolk);
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
        assert_eq!(chardat.customize.tribe, Tribe::Rava);
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
