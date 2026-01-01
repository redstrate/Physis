// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::Ordering;
use std::cmp::Ordering::{Greater, Less};
use std::path::{Path, PathBuf};

use crate::common::{Platform, read_version};
use crate::repository::RepositoryType::{Base, Expansion};
use crate::resource::SqPackRelease;

/// The type of repository, discerning game data from expansion data.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[repr(C)]
pub enum RepositoryType {
    /// The base game directory, like "ffxiv".
    Base,
    /// An expansion directory, like "ex1".
    Expansion {
        /// The expansion number starting at 1.
        number: i32,
    },
}

/// Encapsulates a directory of game data, such as "ex1". This data is also versioned.
/// This handles calculating the correct dat and index filenames, mainly for `GameData`.
#[derive(Debug, Clone, Eq)]
pub struct Repository {
    /// The folder name, such as "ex1".
    pub name: String,
    platform: Platform,
    release: SqPackRelease,
    /// The type of repository, such as "base game" or "expansion".
    pub repo_type: RepositoryType,
    /// The version of the game data.
    pub version: Option<String>,
}

impl PartialEq for Repository {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Ord for Repository {
    fn cmp(&self, other: &Self) -> Ordering {
        // This ensures that the ordering of the repositories is always ffxiv, ex1, ex2 and so on.
        match self.repo_type {
            Base => Less,
            Expansion { number } => {
                let super_number = number;
                match other.repo_type {
                    Base => Greater,
                    Expansion { number } => super_number.cmp(&number),
                }
            }
        }
    }
}

impl PartialOrd for Repository {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// This refers to the specific root directory a file is located in.
/// This is a fixed list of directories, and all of them are known.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Category {
    /// Common files such as game fonts, and other data that doesn't really fit anywhere else.
    Common = 0x00,
    /// Shared data between game maps.
    BackgroundCommon = 0x01,
    /// Game map data such as models, textures, and so on.
    Background = 0x02,
    /// Cutscene content such as animations.
    Cutscene = 0x03,
    /// Character model files and more.
    Character = 0x04,
    /// Compiled shaders used by the retail client.
    Shader = 0x05,
    /// UI layouts and textures.
    UI = 0x06,
    /// Sound effects, basically anything not under `Music`.
    Sound = 0x07,
    /// This "VFX" means "visual effects", and contains textures and definitions for stuff like battle effects.
    VFX = 0x08,
    /// A leftover from 1.0, where the UI was primarily driven by LUA scripts.
    UIScript = 0x09,
    /// Excel data.
    EXD = 0x0A,
    /// Many game events are driven by LUA scripts, such as cutscenes.
    GameScript = 0x0B,
    /// Music!
    Music = 0x0C,
    /// Unknown purpose, most likely to test SqPack functionality.
    SqPackTest = 0x12,
    /// Unknown purpose, most likely debug files.
    Debug = 0x13,
}

pub fn string_to_category(string: &str) -> Option<Category> {
    use crate::repository::Category::*;

    match string {
        "common" => Some(Common),
        "bgcommon" => Some(BackgroundCommon),
        "bg" => Some(Background),
        "cut" => Some(Cutscene),
        "chara" => Some(Character),
        "shader" => Some(Shader),
        "ui" => Some(UI),
        "sound" => Some(Sound),
        "vfx" => Some(VFX),
        "ui_script" => Some(UIScript),
        "exd" => Some(EXD),
        "game_script" => Some(GameScript),
        "music" => Some(Music),
        "sqpack_test" => Some(SqPackTest),
        "debug" => Some(Debug),
        _ => None,
    }
}

impl Repository {
    /// Creates a new base `Repository`, from an existing directory. This may return `None` if
    /// the directory is invalid, e.g. a version file is missing.
    pub fn from_existing_base(
        platform: Platform,
        release: SqPackRelease,
        dir: &str,
    ) -> Option<Repository> {
        let path = Path::new(dir);
        if path.metadata().is_err() {
            return None;
        }

        let mut d = PathBuf::from(dir);
        d.push("ffxivgame.ver");

        let version = read_version(d.as_path());
        Some(Repository {
            name: "ffxiv".to_string(),
            platform,
            release,
            repo_type: Base,
            version,
        })
    }

    /// Creates a new expansion `Repository`, from an existing directory. This may return `None` if
    /// the directory is invalid, e.g. a version file is missing.
    pub fn from_existing_expansion(
        platform: Platform,
        release: SqPackRelease,
        dir: &str,
    ) -> Option<Repository> {
        let path = Path::new(dir);
        if path.metadata().is_err() {
            return None;
        }

        let name = String::from(path.file_stem()?.to_str()?);
        let expansion_number = name[2..3].parse().ok()?;

        let mut d = PathBuf::from(dir);
        d.push(format!("{name}.ver"));

        Some(Repository {
            name,
            platform,
            release,
            repo_type: Expansion {
                number: expansion_number,
            },
            version: read_version(d.as_path()),
        })
    }

    /// Calculate an index filename for a specific category, like _"0a0000.win32.index"_.
    pub fn index_filename(&self, chunk: u8, category: Category) -> String {
        format!(
            "{:02x}{:02}{:02}.{}{}.index",
            category as i32,
            self.expansion(),
            chunk,
            self.platform.shortname(),
            self.release.suffix(),
        )
    }

    /// Calculate an index2 filename for a specific category, like _"0a0000.win32.index"_.
    pub fn index2_filename(&self, chunk: u8, category: Category) -> String {
        format!("{}2", self.index_filename(chunk, category))
    }

    fn expansion(&self) -> i32 {
        match self.repo_type {
            Base => 0,
            Expansion { number } => number,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::Platform;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_base() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("ffxiv");

        let repository = Repository::from_existing_base(
            Platform::Win32,
            SqPackRelease::Retail,
            d.to_str().unwrap(),
        );
        assert!(repository.is_some());
        assert_eq!(repository.unwrap().version.unwrap(), "2012.01.01.0000.0000");
    }

    #[test]
    fn test_expansion() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("ex1");

        let repository = Repository::from_existing_expansion(
            Platform::Win32,
            SqPackRelease::Retail,
            d.to_str().unwrap(),
        );
        assert!(repository.is_some());
        assert_eq!(repository.unwrap().version.unwrap(), "2012.01.01.0000.0000");
    }

    #[test]
    fn test_win32_filenames() {
        let repo = Repository {
            name: "ffxiv".to_string(),
            platform: Platform::Win32,
            release: SqPackRelease::Retail,
            repo_type: RepositoryType::Base,
            version: None,
        };

        assert_eq!(
            repo.index_filename(0, Category::Music),
            "0c0000.win32.index"
        );
        assert_eq!(
            repo.index2_filename(0, Category::Music),
            "0c0000.win32.index2"
        );
    }

    #[test]
    fn test_ps3_filenames() {
        let repo = Repository {
            name: "ffxiv".to_string(),
            platform: Platform::PS3,
            release: SqPackRelease::Retail,
            repo_type: RepositoryType::Base,
            version: None,
        };

        assert_eq!(repo.index_filename(0, Category::Music), "0c0000.ps3.index");
        assert_eq!(
            repo.index2_filename(0, Category::Music),
            "0c0000.ps3.index2"
        );
    }

    #[test]
    fn test_ps4_filenames() {
        let repo = Repository {
            name: "ffxiv".to_string(),
            platform: Platform::PS4,
            release: SqPackRelease::Retail,
            repo_type: RepositoryType::Base,
            version: None,
        };

        assert_eq!(repo.index_filename(0, Category::Music), "0c0000.ps4.index");
        assert_eq!(
            repo.index2_filename(0, Category::Music),
            "0c0000.ps4.index2"
        );
    }

    #[test]
    fn test_ps3_debug_filenames() {
        let repo = Repository {
            name: "ffxiv".to_string(),
            platform: Platform::PS3,
            release: SqPackRelease::Debug,
            repo_type: RepositoryType::Base,
            version: None,
        };

        assert_eq!(
            repo.index_filename(0, Category::Music),
            "0c0000.ps3.d.index"
        );
        assert_eq!(
            repo.index2_filename(0, Category::Music),
            "0c0000.ps3.d.index2"
        );
    }
}
