use std::fs;
use std::path::PathBuf;
use crate::gamedata::MemoryBuffer;
use crate::patch::apply_patch;

/// Boot data for FFXIV.
pub struct BootData {
    path : String,

    pub version : String,
}

fn is_valid(path: &str) -> bool {
    let d = PathBuf::from(path);

    if fs::metadata(d.as_path()).is_err() {
        return false;
    }

    true
}

impl BootData {
    /// Reads from an existing boot data location.
    ///
    /// This will return _None_ if the boot directory is not valid, but it does not check the validity
    /// of each individual file.
    ///
    /// # Example
    ///
    /// ```
    /// # use physis::bootdata::BootData;
    /// let boot = BootData::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/boot");
    /// # assert!(boot.is_none())
    /// ```
    pub fn from_existing(directory: &str) -> Option<BootData> {
        match is_valid(directory) {
            true => Some(BootData {
                path: directory.parse().unwrap(),
                version: String::new()
            }),
            false => {
                println!("Boot data is not valid!");
                None
            }
        }
    }

    pub fn apply_patch(&self, patch_path : &str) {
        apply_patch(&self.path, patch_path);
    }
}