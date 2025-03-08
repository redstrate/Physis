// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::Path;

use binrw::binrw;

#[binrw]
#[brw(repr(u8))]
#[repr(u8)]
#[derive(Copy, Clone)]
/// The language the game data is written for. Some of these languages are supported in the Global region.
pub enum Language {
    /// Used for data that is language-agnostic, such as item data.
    None,
    /// Japanese language.
    Japanese,
    /// English language.
    English,
    /// German language.
    German,
    /// French language.
    French,
    /// Chinese (Simplified) language.
    ChineseSimplified,
    /// Chinese (Traditional) language.
    ChineseTraditional,
    /// Korean language.
    Korean,
}

/// Returns the shorthand language code for `language`. For example, English becomes "en".
pub fn get_language_code(lang: &Language) -> &'static str {
    match &lang {
        Language::None => "",
        Language::Japanese => "ja",
        Language::English => "en",
        Language::German => "de",
        Language::French => "fr",
        Language::ChineseSimplified => "chs",
        Language::ChineseTraditional => "cht",
        Language::Korean => "ko",
    }
}

/// The region of the game. Used to denote the region a patch is meant for.
#[binrw]
#[brw(repr = i16)]
#[derive(Debug, PartialEq, Eq)]
pub enum Region {
    /// The global region, used for any region not specified.
    Global = -1,
    /// Korea and China clients.
    KoreaChina = 1,
}

/// Reads a version file.
pub fn read_version(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}

#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Platform {
    /// Windows and macOS
    Win32 = 0x0,
    /// Playstation 3
    PS3 = 0x1,
    /// Playstation 4
    PS4 = 0x2,
    /// Playstation 5
    PS5 = 0x3,
    /// Xbox
    Xbox = 0x4,
}

pub fn get_platform_string(id: &Platform) -> &'static str {
    match &id {
        Platform::Win32 => "win32",
        Platform::PS3 => "ps3",
        Platform::PS4 => "ps4",
        Platform::PS5 => "ps5",
        Platform::Xbox => "lys",
    }
}
