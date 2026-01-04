// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    collections::HashMap,
    fs::{self, DirEntry, ReadDir},
    path::PathBuf,
};

use crate::{
    ByteBuffer, Error, ReadableFile,
    common::{Language, PLATFORM_LIST, Platform, read_version},
    excel::ExcelSheet,
    exh::EXH,
    repository::{Category, Repository, RepositoryType, string_to_category},
    resource::{
        generic_get_all_sheet_names, generic_parsed, generic_read_excel_sheet,
        generic_read_excel_sheet_header,
    },
    sqpack::{Hash, IndexEntry, SqPackData, SqPackIndex},
};

use super::Resource;

/// Possible actions to repair game files
#[derive(Debug, PartialEq)]
#[repr(C)]
pub enum RepairAction {
    /// Indicates a version file is missing for a repository.
    VersionFileMissing,
    /// The version file is missing, but it can be restored via a backup.
    VersionFileCanRestore,
    /// Indicates a version file has extra newlines or spaces.
    VersionFileExtraSpacing,
}

#[derive(Debug, PartialEq)]
/// Possible errors emitted through the repair process
pub enum RepairError<'a> {
    /// Failed to repair a repository
    FailedRepair(&'a Repository),
}

/// Which release these SqPacks are from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum SqPackRelease {
    /// SqPack files that are generally available from retail.
    Retail,
    /// SqPack files usually only available in debug builds, and are suffixed with ".d"
    Debug,
}

impl SqPackRelease {
    /// The filename suffix for this release.
    pub fn suffix(&self) -> &'static str {
        match self {
            SqPackRelease::Retail => "",
            SqPackRelease::Debug => ".d",
        }
    }
}

/// Used to read files from the retail game, in their SqPack-compressed format.
///
/// In most cases, you probably want to use this inside of a `ResourceResolver`.
#[derive(Debug, Clone)]
pub struct SqPackResource {
    /// The game directory to operate on.
    pub game_directory: String,

    /// Repositories in the game directory.
    pub repositories: Vec<Repository>,

    index_files: HashMap<String, SqPackIndex>,

    /// The platform this resource was designed for.
    platform: Platform,

    /// How these SqPack files were released.
    pub release: SqPackRelease,
}

impl SqPackResource {
    /// Creates a new `SqPackResource` that points to data for `platform` and `release` in `directory`.
    ///
    /// This function automatically determines the platform and release kind based on filenames.
    /// If this is a new install and we can't do that, it will default to Win32 Retail.
    pub fn from_existing(directory: &str) -> Self {
        let (platform, release) = Self::determine_platform_release(directory);

        match is_valid(directory) {
            true => {
                let mut data = Self {
                    game_directory: String::from(directory),
                    repositories: vec![],
                    index_files: HashMap::new(),
                    platform,
                    release,
                };
                data.reload_repositories();
                data
            }
            false => {
                // Game data is not valid! Treating it as a new install...
                Self {
                    game_directory: String::from(directory),
                    repositories: vec![],
                    index_files: HashMap::new(),
                    platform,
                    release,
                }
            }
        }
    }

    /// Determines the `Platform` and `SqPackRelease` for a game directory, based on filenames.
    /// Since we assume all installations are valid, and it never makes sense to "mix" platforms or releases this can be done automatically.
    fn determine_platform_release(directory: &str) -> (Platform, SqPackRelease) {
        let mut d = PathBuf::from(directory);
        d.push("sqpack");
        d.push("ffxiv"); // Every installation should have this directory

        if let Ok(repository_paths) = fs::read_dir(d.as_path()) {
            let repository_paths: ReadDir = repository_paths;

            if let Some(file) = repository_paths
                .filter_map(Result::ok)
                .filter(|s| s.file_type().unwrap().is_file())
                .filter(|s| {
                    s.file_name()
                        .to_str()
                        .unwrap_or_default()
                        .ends_with(".index")
                })
                .take(1)
                .next()
            {
                let filename = file.file_name().to_str().unwrap_or_default().to_string();

                for platform in PLATFORM_LIST {
                    if filename.contains(platform.shortname()) {
                        // Then determine if this is a debug SqPack or not
                        if filename.contains(".d") {
                            return (platform, SqPackRelease::Debug);
                        } else {
                            return (platform, SqPackRelease::Retail);
                        }
                    }
                }
            }
        }

        (Platform::Win32, SqPackRelease::Retail)
    }

    fn reload_repositories(&mut self) {
        self.repositories.clear();

        let mut d = PathBuf::from(self.game_directory.as_str());

        // add initial ffxiv directory
        if let Some(base_repository) =
            Repository::from_existing_base(self.platform, self.release, d.to_str().unwrap())
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
                    self.platform,
                    self.release,
                    repository_path.path().to_str().unwrap(),
                ) {
                    self.repositories.push(expansion_repository);
                }
            }
        }

        self.repositories.sort();
    }

    fn get_dat_file(&self, index_path: &str, data_file_id: u32) -> Option<SqPackData> {
        // Remove the index or index2 from the last bit of the path
        let dat_path = index_path.replace(".index2", "");
        let dat_path = dat_path.replace(".index", "");

        // Append the new dat extension
        let dat_path = format!("{dat_path}.dat{data_file_id}",);

        SqPackData::from_existing(self.platform, &dat_path)
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

    fn get_index_filenames(&self, path: &str) -> Option<Vec<String>> {
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

            index_filenames.push(index_path.into_os_string().into_string().unwrap());

            let index2_path: PathBuf = [
                &self.game_directory,
                "sqpack",
                &repository.name,
                &repository.index2_filename(chunk, category),
            ]
            .iter()
            .collect();

            index_filenames.push(index2_path.into_os_string().into_string().unwrap());
        }

        Some(index_filenames)
    }

    /// Detects whether or not the game files need a repair, right now it only checks for invalid
    /// version files.
    /// If the repair is needed, a list of invalid repositories is given.
    pub fn needs_repair(&self) -> Option<Vec<(&Repository, RepairAction)>> {
        let mut repositories: Vec<(&Repository, RepairAction)> = Vec::new();
        for repository in &self.repositories {
            if let Some(version) = &repository.version {
                let trimmed_version = version.trim();
                if trimmed_version != version {
                    repositories.push((repository, RepairAction::VersionFileExtraSpacing));
                }
            } else {
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
            let ver_path: PathBuf = match repository.repo_type {
                RepositoryType::Base => [self.game_directory.clone(), "ffxivgame.ver".to_string()]
                    .iter()
                    .collect(),
                RepositoryType::Expansion { .. } => [
                    self.game_directory.clone(),
                    "sqpack".to_string(),
                    repository.name.clone(),
                    format!("{}.ver", repository.name),
                ]
                .iter()
                .collect(),
            };

            // TODO: handle ffxivgame base here (except for extra spacing, which does):
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
                RepairAction::VersionFileExtraSpacing => {
                    let ver_file = std::fs::read_to_string(&ver_path).unwrap();
                    ver_file.trim().to_string()
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
            && let Some(index_file) = SqPackIndex::from_existing(self.platform, filename)
        {
            self.index_files.insert(filename.to_string(), index_file);
        }
    }

    fn get_index_file(&self, filename: &str) -> Option<&SqPackIndex> {
        self.index_files.get(filename)
    }

    /// Finds the index entry for `path`, if it exists. Also returns the path of the index file it was found in.
    fn find_entry(&mut self, path: &str) -> Option<(IndexEntry, String)> {
        let index_paths = self.get_index_filenames(path)?;

        for index_path in index_paths {
            self.cache_index_file(&index_path);

            if let Some(index_file) = self.get_index_file(&index_path)
                && let Some(entry) = index_file.find_entry(path)
            {
                return Some((entry, index_path));
            }
        }

        None
    }

    /// Tries to find and preload all available index files.
    /// This is useful if you absolutely do not want to pay the overhead cost on each cache miss when looking up a new file.
    pub fn preload_index_files(&mut self) {
        fn list_files(vec: &mut Vec<PathBuf>, path: &PathBuf) -> std::io::Result<()> {
            if path.is_dir() {
                let paths = fs::read_dir(path)?;
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

    /// Reads a file based on an index hash and the index file you want to read from.
    pub fn read_from_hash(&mut self, index_path: &str, hash: Hash) -> Option<ByteBuffer> {
        self.cache_index_file(index_path);
        let index_file = self.get_index_file(index_path)?;

        let slice = index_file.find_entry_from_hash(hash);
        match slice {
            Some(entry) => {
                let mut dat_file = self.get_dat_file(index_path, entry.data_file_id.into())?;
                dat_file.read_from_offset(entry.offset)
            }
            None => None,
        }
    }

    /// Generically parse a file from a `Resource`.
    pub fn parsed<F: ReadableFile>(&mut self, path: &str) -> Result<F, Error> {
        generic_parsed(self, path)
    }

    /// Read an excel sheet header by name (e.g. "Achievement").
    pub fn read_excel_sheet_header(&mut self, name: &str) -> Result<EXH, Error> {
        generic_read_excel_sheet_header(self, name)
    }

    /// Read an excel sheet by name (e.g. "Achievement").
    pub fn read_excel_sheet(
        &mut self,
        exh: &EXH,
        name: &str,
        language: Language,
    ) -> Result<ExcelSheet, Error> {
        generic_read_excel_sheet(self, exh, name, language)
    }

    /// Returns all known sheet names listed in the root list.
    pub fn get_all_sheet_names(&mut self) -> Result<Vec<String>, Error> {
        generic_get_all_sheet_names(self)
    }
}

impl Resource for SqPackResource {
    fn read(&mut self, path: &str) -> Option<ByteBuffer> {
        let slice = self.find_entry(path);
        match slice {
            Some((entry, index_path)) => {
                let mut dat_file = self.get_dat_file(&index_path, entry.data_file_id.into())?;

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

    fn platform(&self) -> Platform {
        self.platform
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

        SqPackResource::from_existing(d.to_str().unwrap())
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

    #[test]
    fn repository_repair_okay() {
        let mut d = PathBuf::from(std::env::temp_dir());
        d.push("test_sqpack");

        if d.exists() {
            std::fs::remove_dir_all(&d).unwrap();
        }

        std::fs::create_dir_all(&d).unwrap();

        let mut sqpack = d.clone();
        sqpack.push("ffxivgame.ver");
        std::fs::write(sqpack, "2023.09.15.0000.0000").unwrap();

        let resource = SqPackResource::from_existing(d.to_str().unwrap());
        let repo = resource.repositories.first().unwrap();

        assert_eq!(repo.name, "ffxiv");
        assert_eq!(repo.version, Some("2023.09.15.0000.0000".to_string()));
        assert_eq!(resource.needs_repair(), None);
    }

    #[test]
    fn repository_repair_extra_spacing() {
        let mut d = PathBuf::from(std::env::temp_dir());
        d.push("test_sqpack_bad");

        if d.exists() {
            std::fs::remove_dir_all(&d).unwrap();
        }

        std::fs::create_dir_all(&d).unwrap();

        let mut sqpack = d.clone();
        sqpack.push("ffxivgame.ver");
        std::fs::write(&sqpack, "2023.09.15.0000.0000\r\n").unwrap();

        let resource = SqPackResource::from_existing(d.to_str().unwrap());
        let repo = resource.repositories.first().unwrap();

        assert_eq!(repo.name, "ffxiv");
        assert_eq!(repo.version, Some("2023.09.15.0000.0000\r\n".to_string()));

        let repairs = resource.needs_repair();

        assert_eq!(
            repairs,
            Some(vec![(
                resource.repositories.first().unwrap(),
                RepairAction::VersionFileExtraSpacing
            )])
        );

        resource.perform_repair(&repairs.unwrap()).unwrap();

        assert_eq!(
            std::fs::read_to_string(sqpack).unwrap(),
            "2023.09.15.0000.0000"
        );
    }

    #[test]
    fn repository_platform_detection() {
        let test_cases = [
            ("win32_retail", (Platform::Win32, SqPackRelease::Retail)),
            ("win32_debug", (Platform::Win32, SqPackRelease::Debug)),
            ("ps3_retail", (Platform::PS3, SqPackRelease::Retail)),
            ("ps4_retail", (Platform::PS4, SqPackRelease::Retail)),
            ("ps5_retail", (Platform::PS5, SqPackRelease::Retail)),
            ("lys_retail", (Platform::Xbox, SqPackRelease::Retail)),
        ];

        for (path, (platform, release)) in test_cases {
            let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            d.push("resources/tests");
            d.push("platforms");
            d.push(path);

            let resource = SqPackResource::from_existing(d.to_str().unwrap());
            assert_eq!(resource.platform, platform);
            assert_eq!(resource.release, release);
        }
    }
}
