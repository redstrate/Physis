// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::Path;

use binrw::{Endian, binrw};

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
#[brw(repr = u16)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Region {
    /// The global region, used for any region not specified.
    Global = -1,
    /// Korea and China clients.
    KoreaChina = 1,
}

/// Reads a version file. It intentionally reads whitespace as the game reads those characters too.
// TODO: use version type
pub fn read_version(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}

/// Platform used for game data.
#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
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

/// Used internally for automatic SqPack detection. Please update when adding new platforms!
pub(crate) const PLATFORM_LIST: [Platform; 5] = [
    Platform::Win32,
    Platform::PS3,
    Platform::PS4,
    Platform::PS5,
    Platform::Xbox,
];

impl Platform {
    /// Returns the short-hand codename for this platform.
    ///
    /// For example, `Platform::Win32` becomes "win32".
    pub fn shortname(&self) -> &'static str {
        match self {
            Platform::Win32 => "win32",
            Platform::PS3 => "ps3",
            Platform::PS4 => "ps4",
            Platform::PS5 => "ps5",
            Platform::Xbox => "lys",
        }
    }

    /// Returns the endianness for this platform.
    pub(crate) fn endianness(&self) -> Endian {
        match self {
            Platform::PS3 => Endian::Big,
            _ => Endian::Little,
        }
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

/// A file that can be parsed from its serialized byte form.
///
/// This should be implemented for all types readable from SqPack.
pub trait ReadableFile: Sized {
    /// Read an existing file.
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self>;
}

/// A file that can be written back to its serialized byte form.
///
/// This should be implemented for all types readable from SqPack, on a best-effort basis.
pub trait WritableFile: Sized {
    /// Writes data back to a buffer.
    fn write_to_buffer(&self, platform: Platform) -> Option<ByteBuffer>;
}

/// Used for basic sanity checking tests in other modules.
#[cfg(test)]
pub fn pass_random_invalid<T: ReadableFile>() {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests");
    d.push("random");

    // Feeding it invalid data should not panic
    // Note that we don't check the Option currently, because some types like Hwc return Some regardless.
    T::from_existing(
        Platform::Win32,
        &std::fs::read(d).expect("Could not read random test file"),
    );
}

/// A continuous block of memory which is not owned, and comes either from an in-memory location or from a file.
pub type ByteSpan<'a> = &'a [u8];

/// A continuous block of memory which is owned.
pub type ByteBuffer = Vec<u8>;

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

    #[test]
    fn test_version() {
        let mut dir = std::env::temp_dir();
        dir.push("test.ver");
        if dir.exists() {
            std::fs::remove_file(&dir).unwrap();
        }

        assert_eq!(read_version(&dir), None);

        std::fs::write(&dir, "2023.09.15.0000.0000").unwrap();
        assert_eq!(read_version(&dir), Some("2023.09.15.0000.0000".to_string()));

        std::fs::write(&dir, "2023.09.15.0000.0000\r\n").unwrap();
        assert_eq!(
            read_version(&dir),
            Some("2023.09.15.0000.0000\r\n".to_string())
        );
    }
}
