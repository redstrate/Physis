// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::PathBuf;

use crate::patch::{PatchError, ZiPatch};

/// Represents the boot data for FFXIV, which is located under the "boot" directory.
pub struct BootData {
    path: String,

    /// The current version of the boot data, e.g. "2012.01.01.0000.0000".
    pub version: String,
}

impl BootData {
    /// Reads from an existing boot data location.
    ///
    /// This will return a BootData even if the game directory is technically
    /// invalid, but it won't have a valid version.
    ///
    /// # Example
    ///
    /// ```
    /// # use physis::bootdata::BootData;
    /// let boot = BootData::from_existing("SquareEnix/Final Fantasy XIV - A Realm Reborn/boot");
    /// ```
    pub fn from_existing(directory: &str) -> BootData {
        match Self::is_valid(directory) {
            true => BootData {
                path: directory.parse().ok().unwrap(),
                version: fs::read_to_string(format!("{directory}/ffxivboot.ver"))
                    .ok()
                    .unwrap(),
            },
            false => {
                // Boot data is not valid! Returning one anyway, but without a version.
                BootData {
                    path: directory.parse().ok().unwrap(),
                    version: String::default(),
                }
            }
        }
    }

    /// Applies the patch to boot data and returns any errors it encounters. This function will not update the version in the BootData struct.
    pub fn apply_patch(&self, patch_path: &str) -> Result<(), PatchError> {
        ZiPatch::apply(&self.path, patch_path)
    }

    fn is_valid(path: &str) -> bool {
        let mut d = PathBuf::from(path);

        // Check if directory exists
        if fs::metadata(d.as_path()).is_err() {
            return false;
        }

        // Check if it has a version file
        d.push("ffxivboot.ver");
        if fs::metadata(d.as_path()).is_err() {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_boot_dir() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("valid_boot");

        let boot_data = BootData::from_existing(d.as_path().to_str().unwrap());
        assert_eq!(boot_data.version, "2012.01.01.0000.0000");
    }

    #[test]
    fn test_invalid_boot_dir() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("invalid_boot"); // intentionally missing so it doesn't have a .ver

        let boot_data = BootData::from_existing(d.as_path().to_str().unwrap());
        assert_eq!(boot_data.version, "");
    }
}
