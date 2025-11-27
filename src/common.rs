// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::Path;

use binrw::binrw;

#[binrw]
#[brw(repr(u8))]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Language the game data is written for.
pub enum Language {
    /// Used for data that is language-agnostic.
    None,
    /// Japanese language. Only available in the Global client.
    Japanese,
    /// English language. Only available in the Global client.
    English,
    /// German language. Only available in the Global client.
    German,
    /// French language. Only available in the Global client.
    French,
    /// Chinese (Simplified) language. Only available in the Chinese client.
    ChineseSimplified,
    /// Chinese (Traditional) language. Only available in the Chinese client.
    ChineseTraditional,
    /// Korean language. Only available in the Korean client.
    Korean,
}

/// Returns the shorthand language code for `language`.
///
/// For example, English becomes "en".
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
// TODO: use version type
pub fn read_version(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}

/// Platform used for game data.
#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Platform {
    /// Windows and macOS.
    Win32 = 0x0,
    /// Playstation 3.
    PS3 = 0x1,
    /// Playstation 4.
    PS4 = 0x2,
    /// Playstation 5.
    PS5 = 0x3,
    /// Xbox One.
    Xbox = 0x4,
}

/// Returns the short-hand version of `id`.
///
/// For example, `Platform::Win32` becomes "win32".
pub fn get_platform_string(id: &Platform) -> &'static str {
    match &id {
        Platform::Win32 => "win32",
        Platform::PS3 => "ps3",
        Platform::PS4 => "ps4",
        Platform::PS5 => "ps5",
        Platform::Xbox => "lys",
    }
}

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

/// Represents a game version, e.g. "2025.02.27.0000.0000".
///
/// Unlike a normal string, this can sort itself in a sensible way.
#[derive(PartialEq, Eq, PartialOrd)]
pub struct Version<'a>(pub &'a str);

#[derive(PartialEq, Eq, Ord, PartialOrd)]
struct VersionParts {
    year: i32,
    month: i32,
    day: i32,
    patch1: i32,
    patch2: i32,
}

impl VersionParts {
    fn new(version: &str) -> Self {
        let parts: Vec<&str> = version.split('.').collect();

        Self {
            year: parts[0].parse::<i32>().unwrap(),
            month: parts[1].parse::<i32>().unwrap(),
            day: parts[2].parse::<i32>().unwrap(),
            patch1: parts[3].parse::<i32>().unwrap(),
            patch2: parts[4].parse::<i32>().unwrap(),
        }
    }
}

impl Display for Version<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Ord for Version<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let our_version_parts = VersionParts::new(self.0);
        let their_version_parts = VersionParts::new(other.0);

        our_version_parts.cmp(&their_version_parts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert!(Version("2025.02.27.0000.0000") == Version("2025.02.27.0000.0000"));
        assert!(Version("2025.01.20.0000.0000") != Version("2025.02.27.0000.0000"));
    }

    #[test]
    fn test_ordering() {
        // year
        assert!(Version("2025.02.27.0000.0000") > Version("2024.02.27.0000.0000"));

        // month
        assert!(Version("2025.03.27.0000.0000") > Version("2025.02.27.0000.0000"));

        // day
        assert!(Version("2025.02.28.0000.0000") > Version("2025.02.27.0000.0000"));

        // patch1
        assert!(Version("2025.02.27.1000.0000") > Version("2025.02.27.0000.0000"));

        // patch2
        assert!(Version("2025.02.27.0000.1000") > Version("2025.02.27.0000.0000"));
    }
}
