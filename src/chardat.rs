use crate::gamedata::MemoryBuffer;
use crate::sha1::Sha1;
use binrw::{binread, binrw};
use binrw::BinRead;
use std::fs::read;
use std::io::{Cursor, Seek, SeekFrom};
use std::ops::Sub;
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

fn convert_dat_gender(x: u8) -> Gender {
    match x {
        0 => Gender::Male,
        1 => Gender::Female,
        _ => Gender::Male
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

#[binread]
#[br(little)]
#[repr(C)]
#[br(magic = 0x2013FF14u32)]
#[derive(Debug)]
pub struct CharDat { // version 4
    pub version: u32,
    #[br(pad_after = 4)]
    pub checksum: u32,

    #[br(map = | x: u8 | convert_dat_race(x) )]
    pub race: Race,
    #[br(map = | x: u8 | convert_dat_gender(x) )]
    pub gender: Gender,
    pub age: u8, // Normal = 1, Old = 3, Young = 4
    pub height: u8,
    #[br(map = | x: u8 | convert_dat_subrace(x) )]
    pub subrace: Subrace,
    pub head: u8,
    pub hair: u8,
    #[br(map = | x: u8 | x != 0 )]
    pub enable_highlights: bool,
    pub skin_tone: u8,
    pub right_eye_color: u8,
    pub hair_tone: u8,
    pub highlights: u8,
    pub facial_features: u8,
    pub limbal_eyes: u8,
    pub eyebrows: u8,
    pub left_eye_color: u8,
    pub eyes: u8,
    pub nose: u8,
    pub jaw: u8,
    pub mouth: u8,
    pub lips_tone_fur_pattern: u8,
    pub tail: u8,
    pub face_paint: u8,
    pub bust: u8,
    pub face_paint_color: u8,
    pub voice: u8,

    #[br(pad_before = 1)]
    pub timestamp: [u8; 4]
}

impl CharDat {
    /// Parses an existing dat file.
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<CharDat> {
        let mut cursor = Cursor::new(buffer);

        Some(CharDat::read(&mut cursor).ok()?)
    }
}