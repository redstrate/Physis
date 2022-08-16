use std::fs;
use std::fs::DirEntry;
use std::path::PathBuf;
use crate::common::Language;
use crate::dat::DatFile;
use crate::exd::EXD;
use crate::exh::EXH;
use crate::exl::EXL;
use crate::index::IndexFile;
use crate::patch::{apply_patch, PatchError};
use crate::repository::{Category, Repository, string_to_category};
use crate::sqpack::calculate_hash;

/// Framework for operating on game data.
pub struct GameData {
    /// The game directory to operate on.
    pub game_directory: String,

    /// Repositories in the game directory.
    pub repositories: Vec<Repository>,
}

fn is_valid(path: &str) -> bool {
    let d = PathBuf::from(path);

    if fs::metadata(d.as_path()).is_err() {
        println!("Failed game directory.");
        return false;
    }

    true
}

pub type MemoryBuffer = Vec<u8>;

impl GameData {
    /// Read game data from an existing game installation.
    ///
    /// This will return _None_ if the game directory is not valid, but it does not check the validity
    /// of each individual file.
    ///
    /// **Note**: None of the repositories are searched, and it's required to call `reload_repositories()`.
    ///
    /// # Example
    ///
    /// ```
    /// # use physis::gamedata::GameData;
    /// GameData::from_existing("$FFXIV/game");
    /// ```
    pub fn from_existing(directory: &str) -> Option<GameData> {
        match is_valid(directory) {
            true => Some(Self {
                game_directory: String::from(directory),
                repositories: vec![],
            }),
            false => {
                println!("Game data is not valid!");
                None
            }
        }
    }

    /// Reloads all repository information from disk. This is a fast operation, as it's not actually
    /// reading any dat files yet.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::gamedata::GameData;
    /// let mut game = GameData::from_existing("$FFXIV/game").unwrap();
    /// game.reload_repositories();
    /// ```
    pub fn reload_repositories(&mut self) {
        self.repositories.clear();

        let mut d = PathBuf::from(self.game_directory.as_str());
        d.push("sqpack");

        let repository_paths: Vec<DirEntry> = fs::read_dir(d.as_path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|s| s.file_type().unwrap().is_dir())
            .collect();

        for repository_path in repository_paths {
            self.repositories.push(Repository::from_existing(repository_path.path().to_str().unwrap()).unwrap());
        }

        self.repositories.sort();
    }

    fn get_index_file(&self, path: &str) -> Option<IndexFile> {
        let (repository, category) = self.parse_repository_category(path).unwrap();

        let index_path : PathBuf = [self.game_directory.clone(),
            "sqpack".to_string(),
            repository.name.clone(),
            repository.index_filename(category)]
            .iter().collect();

        IndexFile::from_existing(index_path.to_str()?)
    }

    fn get_dat_file(&self, path: &str, data_file_id : u32) -> Option<DatFile> {
        let (repository, category) = self.parse_repository_category(path).unwrap();

        let dat_path : PathBuf = [self.game_directory.clone(),
            "sqpack".to_string(),
            repository.name.clone(),
            repository.dat_filename(category, data_file_id)]
            .iter().collect();

        DatFile::from_existing(dat_path.to_str()?)
    }

    /// Checks if a file located at `path` exists.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::gamedata::GameData;
    /// # let mut game = GameData::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/game").unwrap();
    /// if game.exists("exd/cid.exl") {
    ///     println!("Cid really does exist!");
    /// } else {
    ///     println!("Oh noes!");
    /// }
    /// ```
    pub fn exists(&self, path: &str) -> bool {
        let hash = calculate_hash(path);

        let index_file = self.get_index_file(path)
            .expect("Failed to find index file.");

        index_file.entries.iter().any(|s| s.hash == hash)
    }

    /// Extracts the file located at `path`. This is returned as an in-memory buffer, and will usually
    /// have to be further parsed.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// # use physis::gamedata::GameData;
    /// # use std::io::Write;
    /// # let mut game = GameData::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/game").unwrap();
    /// let data = game.extract("exd/root.exl").unwrap();
    ///
    /// let mut file = std::fs::File::create("root.exl").unwrap();
    /// file.write(data.as_slice()).unwrap();
    /// ```
    pub fn extract(&self, path: &str) -> Option<MemoryBuffer> {
        let hash = calculate_hash(path);

        let index_file = self.get_index_file(path)?;

        let slice = index_file.entries.iter().find(|s| s.hash == hash);
        match slice {
            Some(entry) => {
                let mut dat_file = self.get_dat_file(path, entry.bitfield.data_file_id())?;

                dat_file.read_from_offset(entry.bitfield.offset())
            }
            None => None
        }
    }

    /// Parses a path structure and spits out the corresponding category and repository.
    fn parse_repository_category(&self, path: &str) -> Option<(&Repository, Category)> {
        let tokens: Vec<&str> = path.split('/').collect(); // TODO: use split_once here
        let repository_token = tokens[0];

        if tokens.len() < 2 {
            return None;
        }

        for repository in &self.repositories {
            if repository.name == repository_token {
                return Some((repository, string_to_category(tokens[1])?));
            }
        }

        Some((&self.repositories[0], string_to_category(tokens[0])?))
    }

    pub fn read_excel_sheet_header(&self, name : &str) -> Option<EXH> {
        let root_exl_file = self.extract("exd/root.exl")?;

        let root_exl = EXL::from_existing(&root_exl_file)?;

        for (row, _) in root_exl.entries {
            if row == name {
                let new_filename = name.to_lowercase();

                let path = format!("exd/{new_filename}.exh");

                return EXH::from_existing(&self.extract(&path)?)
            }
        }

        None
    }

    pub fn read_excel_sheet(&self, name : &str, exh : &EXH, language : Language, page : usize) -> Option<EXD> {
        let exd_path = format!("exd/{}", EXD::calculate_filename(name, language, &exh.pages[page]));

        let exd_file = self.extract(&exd_path).unwrap();

        EXD::from_existing(exh, &exd_file)
    }

    pub fn apply_patch(&self, patch_path : &str) -> Result<(), PatchError> {
        apply_patch(&self.game_directory, patch_path)
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::Category::EXD;
    use super::*;

    fn common_setup_data() -> GameData {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("valid_sqpack");
        d.push("game");

        GameData::from_existing(d.to_str().unwrap()).unwrap()
    }

    #[test]
    fn repository_ordering() {
        let mut data = common_setup_data();
        data.reload_repositories();

        assert_eq!(data.repositories[0].name, "ffxiv");
        assert_eq!(data.repositories[1].name, "ex1");
        assert_eq!(data.repositories[2].name, "ex2");
    }

    #[test]
    fn repository_and_category_parsing() {
        let mut data = common_setup_data();
        data.reload_repositories();

        assert_eq!(data.parse_repository_category("exd/root.exl").unwrap(),
                   (&data.repositories[0], EXD));
        assert!(data.parse_repository_category("what/some_font.dat").is_none());
    }
}