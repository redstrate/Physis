use std::fs;
use std::path::PathBuf;

/// Boot data for FFXIV.
pub struct BootData {
    version: String,
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
                version: String::new()
            }),
            false => {
                println!("Boot data is not valid!");
                None
            }
        }
    }
}