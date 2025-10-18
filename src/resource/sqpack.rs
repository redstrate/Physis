// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    fs::{self, DirEntry, ReadDir},
    path::PathBuf,
};

use crate::{
    ByteBuffer,
    common::{Platform, read_version},
    patch::{PatchError, ZiPatch},
    repository::{Category, Repository, string_to_category},
    sqpack::{IndexEntry, SqPackData, SqPackIndex},
};

use super::Resource;

/// Possible actions to repair game files
#[derive(Debug)]
pub enum RepairAction {
    /// Indicates a version file is missing for a repository
    VersionFileMissing,
    /// The version file is missing, but it can be restored via a backup
    VersionFileCanRestore,
}

#[derive(Debug)]
/// Possible errors emitted through the repair process
pub enum RepairError<'a> {
    /// Failed to repair a repository
    FailedRepair(&'a Repository),
}

/// Used to read files from the retail game, in their SqPack-compressed format.
pub struct SqPackResource {
    /// The game directory to operate on.
    pub game_directory: String,

    /// Repositories in the game directory.
    pub repositories: Vec<Repository>,

    index_files: HashMap<String, SqPackIndex>,
}

impl SqPackResource {
    pub fn from_existing(platform: Platform, directory: &str) -> Self {
        match is_valid(directory) {
            true => {
                let mut data = Self {
                    game_directory: String::from(directory),
                    repositories: vec![],
                    index_files: HashMap::new(),
                };
                data.reload_repositories(platform);
                data
            }
            false => {
                // Game data is not valid! Treating it as a new install...
                Self {
                    game_directory: String::from(directory),
                    repositories: vec![],
                    index_files: HashMap::new(),
                }
            }
        }
    }

    fn reload_repositories(&mut self, platform: Platform) {
        self.repositories.clear();

        let mut d = PathBuf::from(self.game_directory.as_str());

        // add initial ffxiv directory
        if let Some(base_repository) =
            Repository::from_existing_base(platform.clone(), d.to_str().unwrap())
        {
            self.repositories.push(base_repository);
        }

        // add expansions
        d.push("sqpack");

        if let Ok(repository_paths) = fs::read_dir(d.as_path()) {
            let repository_paths: ReadDir = repository_paths;

            let repository_paths: Vec<DirEntry> = repository_paths
                .filter_map(Result::ok)
                .filter(|s| s.file_type().unwrap().is_dir())
                .collect();

            for repository_path in repository_paths {
                if let Some(expansion_repository) = Repository::from_existing_expansion(
                    platform.clone(),
                    repository_path.path().to_str().unwrap(),
                ) {
                    self.repositories.push(expansion_repository);
                }
            }
        }

        self.repositories.sort();
    }

    fn get_dat_file(&self, path: &str, chunk: u8, data_file_id: u32) -> Option<SqPackData> {
        let (repository, category) = self.parse_repository_category(path).unwrap();

        let dat_path: PathBuf = [
            self.game_directory.clone(),
            "sqpack".to_string(),
            repository.name.clone(),
            repository.dat_filename(chunk, category, data_file_id),
        ]
        .iter()
        .collect();

        SqPackData::from_existing(dat_path.to_str()?)
    }

    /// Finds the offset inside of the DAT file for `path`.
    pub fn find_offset(&mut self, path: &str) -> Option<u64> {
        let slice = self.find_entry(path);
        slice.map(|(entry, _)| entry.offset)
    }

    /// Parses a path structure and spits out the corresponding category and repository.
    fn parse_repository_category(&self, path: &str) -> Option<(&Repository, Category)> {
        if self.repositories.is_empty() {
            return None;
        }

        let tokens: Vec<&str> = path.split('/').collect();

        // Search for expansions
        let repository_token = tokens[1];
        for repository in &self.repositories {
            if repository.name == repository_token {
                return Some((repository, string_to_category(tokens[0])?));
            }
        }

        // Fallback to ffxiv
        Some((&self.repositories[0], string_to_category(tokens[0])?))
    }

    fn get_index_filenames(&self, path: &str) -> Option<Vec<(String, u8)>> {
        let (repository, category) = self.parse_repository_category(path)?;

        let mut index_filenames = vec![];

        for chunk in 0..255 {
            let index_path: PathBuf = [
                &self.game_directory,
                "sqpack",
                &repository.name,
                &repository.index_filename(chunk, category),
            ]
            .iter()
            .collect();

            index_filenames.push((index_path.into_os_string().into_string().unwrap(), chunk));

            let index2_path: PathBuf = [
                &self.game_directory,
                "sqpack",
                &repository.name,
                &repository.index2_filename(chunk, category),
            ]
            .iter()
            .collect();

            index_filenames.push((index2_path.into_os_string().into_string().unwrap(), chunk));
        }

        Some(index_filenames)
    }

    /// Applies the patch to game data and returns any errors it encounters. This function will not update the version in the GameData struct.
    pub fn apply_patch(&self, patch_path: &str) -> Result<(), PatchError> {
        ZiPatch::apply(&self.game_directory, patch_path)
    }

    /// Detects whether or not the game files need a repair, right now it only checks for invalid
    /// version files.
    /// If the repair is needed, a list of invalid repositories is given.
    pub fn needs_repair(&self) -> Option<Vec<(&Repository, RepairAction)>> {
        let mut repositories: Vec<(&Repository, RepairAction)> = Vec::new();
        for repository in &self.repositories {
            if repository.version.is_none() {
                // Check to see if a .bck file is created, as we might be able to use that
                let ver_bak_path: PathBuf = [
                    self.game_directory.clone(),
                    "sqpack".to_string(),
                    repository.name.clone(),
                    format!("{}.bck", repository.name),
                ]
                .iter()
                .collect();

                let repair_action = if read_version(&ver_bak_path).is_some() {
                    RepairAction::VersionFileCanRestore
                } else {
                    RepairAction::VersionFileMissing
                };

                repositories.push((repository, repair_action));
            }
        }

        if repositories.is_empty() {
            None
        } else {
            Some(repositories)
        }
    }

    /// Performs the repair, assuming any damaging effects it may have
    /// Returns true only if all actions were taken are successful.
    /// NOTE: This is a destructive operation, especially for InvalidVersion errors.
    pub fn perform_repair<'a>(
        &self,
        repositories: &Vec<(&'a Repository, RepairAction)>,
    ) -> Result<(), RepairError<'a>> {
        for (repository, action) in repositories {
            let ver_path: PathBuf = [
                self.game_directory.clone(),
                "sqpack".to_string(),
                repository.name.clone(),
                format!("{}.ver", repository.name),
            ]
            .iter()
            .collect();

            let new_version: String = match action {
                RepairAction::VersionFileMissing => {
                    let repo_path: PathBuf = [
                        self.game_directory.clone(),
                        "sqpack".to_string(),
                        repository.name.clone(),
                    ]
                    .iter()
                    .collect();

                    fs::remove_dir_all(&repo_path)
                        .ok()
                        .ok_or(RepairError::FailedRepair(repository))?;

                    fs::create_dir_all(&repo_path)
                        .ok()
                        .ok_or(RepairError::FailedRepair(repository))?;

                    "2012.01.01.0000.0000".to_string() // TODO: is this correct for expansions?
                }
                RepairAction::VersionFileCanRestore => {
                    let ver_bak_path: PathBuf = [
                        self.game_directory.clone(),
                        "sqpack".to_string(),
                        repository.name.clone(),
                        format!("{}.bck", repository.name),
                    ]
                    .iter()
                    .collect();

                    read_version(&ver_bak_path).ok_or(RepairError::FailedRepair(repository))?
                }
            };

            fs::write(ver_path, new_version)
                .ok()
                .ok_or(RepairError::FailedRepair(repository))?;
        }

        Ok(())
    }

    fn cache_index_file(&mut self, filename: &str) {
        if !self.index_files.contains_key(filename)
            && let Some(index_file) = SqPackIndex::from_existing(filename)
        {
            self.index_files.insert(filename.to_string(), index_file);
        }
    }

    fn get_index_file(&self, filename: &str) -> Option<&SqPackIndex> {
        self.index_files.get(filename)
    }

    fn find_entry(&mut self, path: &str) -> Option<(IndexEntry, u8)> {
        let index_paths = self.get_index_filenames(path)?;

        for (index_path, chunk) in index_paths {
            self.cache_index_file(&index_path);

            if let Some(index_file) = self.get_index_file(&index_path)
                && let Some(entry) = index_file.find_entry(path)
            {
                return Some((entry, chunk));
            }
        }

        None
    }

    /// Tries to find and preload all available index files.
    /// This is useful if you absolutely do not want to pay the overhead cost on each cache miss when looking up a new file.
    pub fn preload_index_files(&mut self) {
        fn list_files(vec: &mut Vec<PathBuf>, path: &PathBuf) -> std::io::Result<()> {
            if path.is_dir() {
                let paths = fs::read_dir(&path)?;
                for path_result in paths {
                    let full_path = path_result?.path();
                    let _ = list_files(vec, &full_path);
                }
            } else {
                vec.push(path.clone());
            }
            Ok(())
        }

        let mut index_paths = Vec::new();
        let _ = list_files(&mut index_paths, &PathBuf::from(&self.game_directory));

        for index_path in index_paths {
            if index_path.extension().unwrap_or_default() == "index"
                || index_path.extension().unwrap_or_default() == "index2"
            {
                self.cache_index_file(&index_path.into_os_string().into_string().unwrap());
            }
        }
    }
}

impl Resource for SqPackResource {
    fn read(&mut self, path: &str) -> Option<ByteBuffer> {
        let slice = self.find_entry(path);
        match slice {
            Some((entry, chunk)) => {
                let mut dat_file = self.get_dat_file(path, chunk, entry.data_file_id.into())?;

                dat_file.read_from_offset(entry.offset)
            }
            None => None,
        }
    }

    fn exists(&mut self, path: &str) -> bool {
        let Some(_) = self.get_index_filenames(path) else {
            return false;
        };

        self.find_entry(path).is_some()
    }
}

fn is_valid(path: &str) -> bool {
    let d = PathBuf::from(path);

    if fs::metadata(d.as_path()).is_err() {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use crate::repository::Category::*;

    use super::*;

    fn common_setup_data() -> SqPackResource {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("valid_sqpack");
        d.push("game");

        SqPackResource::from_existing(Platform::Win32, d.to_str().unwrap())
    }

    #[test]
    fn repository_ordering() {
        let data = common_setup_data();

        assert_eq!(data.repositories[0].name, "ffxiv");
        assert_eq!(data.repositories[1].name, "ex1");
        assert_eq!(data.repositories[2].name, "ex2");
    }

    #[test]
    fn repository_and_category_parsing() {
        let data = common_setup_data();

        // fallback to ffxiv
        assert_eq!(
            data.parse_repository_category("exd/root.exl").unwrap(),
            (&data.repositories[0], EXD)
        );
        // ex1
        assert_eq!(
            data.parse_repository_category("bg/ex1/01_roc_r2/twn/r2t1/level/planevent.lgb")
                .unwrap(),
            (&data.repositories[1], Background)
        );
        // ex2
        assert_eq!(
            data.parse_repository_category("bg/ex2/01_gyr_g3/fld/g3fb/level/planner.lgb")
                .unwrap(),
            (&data.repositories[2], Background)
        );
        // invalid but should still parse I guess
        assert!(
            data.parse_repository_category("what/some_font.dat")
                .is_none()
        );
    }
}
