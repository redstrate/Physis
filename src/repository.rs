// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cmp::Ordering;
use std::cmp::Ordering::{Greater, Less};
use std::path::{Path, PathBuf};
use binrw::binrw;

use crate::common::{Platform, read_version};
use crate::repository::RepositoryType::{Base, Expansion};

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
#[derive(Debug, Clone)]
pub struct Repository {
    /// The folder name, such as "ex1".
    pub name: String,
    /// The platform this repository is designed for.
    pub platform: Platform,
    /// The type of repository, such as "base game" or "expansion".
    pub repo_type: RepositoryType,
    /// The version of the game data.
    pub version: Option<String>,
}

impl Eq for Repository {}

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
    Common,
    /// Shared data between game maps.
    BackgroundCommon,
    /// Game map data such as models, textures, and so on.
    Background,
    /// Cutscene content such as animations.
    Cutscene,
    /// Character model files and more.
    Character,
    /// Compiled shaders used by the retail client.
    Shader,
    /// UI layouts and textures.
    UI,
    /// Sound effects, basically anything not under `Music`.
    Sound,
    /// This "VFX" means "visual effects", and contains textures and definitions for stuff like battle effects.
    VFX,
    /// A leftover from 1.0, where the UI was primarily driven by LUA scripts.
    UIScript,
    /// Excel data.
    EXD,
    /// Many game events are driven by LUA scripts, such as cutscenes.
    GameScript,
    /// Music!
    Music,
    /// Unknown purpose, most likely to test SqPack functionality.
    SqPackTest,
    /// Unknown purpose, most likely debug files.
    Debug,
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
    pub fn from_existing_base(platform: Platform, dir: &str) -> Option<Repository> {
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
            repo_type: Base,
            version,
        })
    }

    /// Creates a new expansion `Repository`, from an existing directory. This may return `None` if
    /// the directory is invalid, e.g. a version file is missing.
    pub fn from_existing_expansion(platform: Platform, dir: &str) -> Option<Repository> {
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
            repo_type: Expansion {
                number: expansion_number,
            },
            version: read_version(d.as_path()),
        })
    }

    /// Calculate an index filename for a specific category, like _"0a0000.win32.index"_.
    pub fn index_filename(&self, category: Category) -> String {
        format!(
            "{:02x}{:02}{:02}.{}.index",
            category as i32,
            self.expansion(),
            0,
            "win32"
        )
    }
    
    /// Calculate an index2 filename for a specific category, like _"0a0000.win32.index"_.
    pub fn index2_filename(&self, category: Category) -> String {
        format!(
            "{}2",
            self.index_filename(category)
        )
    }

    /// Calculate a dat filename given a category and a data file id, returns something like _"0a0000.win32.dat0"_.
    pub fn dat_filename(&self, category: Category, data_file_id: u32) -> String {
        let expansion = self.expansion();
        let chunk = 0;
        let platform = "win32";

        format!(
            "{:02x}{expansion:02}{chunk:02}.{platform}.dat{data_file_id}",
            category as u32
        )
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
    use std::path::PathBuf;
    use crate::common::Platform;

    use crate::repository::Repository;

    #[test]
    fn test_base() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("ffxiv");

        let repository = Repository::from_existing_base(Platform::Win32, d.to_str().unwrap());
        assert!(repository.is_some());
        assert_eq!(repository.unwrap().version.unwrap(), "2012.01.01.0000.0000");
    }

    #[test]
    fn test_expansion() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("ex1");

        let repository = Repository::from_existing_expansion(Platform::Win32, d.to_str().unwrap());
        assert!(repository.is_some());
        assert_eq!(repository.unwrap().version.unwrap(), "2012.01.01.0000.0000");
    }
}