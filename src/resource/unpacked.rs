// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use crate::ByteBuffer;

use super::Resource;

/// Used to read unpacked files from a directory.
///
/// In most cases, you probably want to use this inside of a `ResourceResolver`.
pub struct UnpackedResource {
    base_directory: String,
}

impl UnpackedResource {
    pub fn from_existing(base_directory: &str) -> Self {
        Self {
            base_directory: base_directory.to_string(),
        }
    }
}

impl Resource for UnpackedResource {
    fn read(&mut self, path: &str) -> Option<ByteBuffer> {
        let mut new_path = PathBuf::from(&self.base_directory);
        new_path.push(path.to_lowercase());

        std::fs::read(new_path).ok()
    }

    fn exists(&mut self, path: &str) -> bool {
        let mut new_path = PathBuf::from(&self.base_directory);
        new_path.push(path.to_lowercase());

        std::fs::exists(new_path).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn common_setup_data() -> UnpackedResource {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");

        UnpackedResource::from_existing(d.to_str().unwrap())
    }

    #[test]
    fn read_files() {
        let mut data = common_setup_data();

        assert!(data.read("empty_planlive.lgb").is_some());
        assert!(data.read("non_existent.lgb").is_none());
    }

    #[test]
    fn exist_files() {
        let mut data = common_setup_data();

        assert_eq!(data.exists("empty_planlive.lgb"), true);
        assert_eq!(data.exists("non_existent.lgb"), false);
    }
}
