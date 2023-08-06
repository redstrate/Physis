// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use binrw::BinRead;
use binrw::binrw;

use crate::gamedata::MemoryBuffer;
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

#[binrw]
#[br(little)]
#[repr(C)]
#[br(magic = 0x2013FF14u32)]
#[derive(Debug)]
pub struct CharDat { // version 4
    pub version: u32,
    #[br(pad_after = 4)]
    pub checksum: u32,

    #[br(map = | x: u8 | convert_dat_race(x) )]
    #[bw(map = | race: &Race | convert_race_dat(race) )]
    pub race: Race,
    #[br(map = | x: u8 | convert_dat_gender(x) )]
    #[bw(map = | gender: &Gender | convert_gender_dat(gender) )]
    pub gender: Gender,
    pub age: u8, // Normal = 1, Old = 3, Young = 4
    pub height: u8,
    #[br(map = | x: u8 | convert_dat_subrace(x) )]
    #[bw(map = | subrace: &Subrace | convert_subrace_dat(subrace) )]
    pub subrace: Subrace,
    pub head: u8,
    pub hair: u8,
    #[br(map = | x: u8 | x != 0 )]
    #[bw(map = | x: &bool | if *x { 1u8 } else { 0u8 } )]
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