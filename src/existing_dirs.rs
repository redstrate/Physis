// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

// Rust deprecating this is stupid, I don't want to use a crate here
#[allow(deprecated)]
use std::env::home_dir;

use std::fs;
use std::fs::read_dir;
use std::path::PathBuf;

use crate::resource::SqPackResource;

/// Where the existing installation came from
#[derive(Clone, Copy)]
#[repr(C)]
pub enum ExistingInstallType {
    /// Installed via the official launcher
    OfficialLauncher,
    /// Installed via XIVQuickLauncher
    XIVQuickLauncher,
    /// Installed via XIVLauncherCore
    XIVLauncherCore,
    /// Installed via XIVOnMac
    XIVOnMac,
    /// Installed via Astra
    Astra,
}

/// An existing install location on disk
pub struct ExistingGameDirectory {
    /// The application where this installation was from
    pub install_type: ExistingInstallType,
    /// The path to the "main folder" where "game" and "boot" sits
    pub path: String,
    /// The latest expansion and game version in this directory.
    pub version: String,
}

fn read_version(path: &str) -> String {
    let mut path = PathBuf::from(path);
    path.push("game");
    let path = path.to_str().unwrap().to_string();

    let game_data = SqPackResource::from_existing(&path);

    if let Some(latest_repository) = game_data.repositories.last() {
        return latest_repository.version.clone().unwrap_or_default();
    }

    String::default()
}

/// Finds existing installations on disk. Will only return locations that actually have files in them, and a really basic check to see if the data is valid.
pub fn find_existing_game_dirs() -> Vec<ExistingGameDirectory> {
    let mut install_dirs = Vec::new();

    match std::env::consts::OS {
        "linux" => {
            // Official install (Steam)
            {
                let path = from_home_dir(
                    ".steam/steam/steamapps/common/FINAL FANTASY XIV - A Realm Reborn",
                );
                install_dirs.push(ExistingGameDirectory {
                    install_type: ExistingInstallType::OfficialLauncher,
                    path: path.clone(),
                    version: read_version(&path),
                });
            }

            // XIVLauncherCore location
            {
                let path = from_home_dir(".xlcore/ffxiv");
                install_dirs.push(ExistingGameDirectory {
                    install_type: ExistingInstallType::XIVLauncherCore,
                    path: path.clone(),
                    version: read_version(&path),
                });
            }

            // Astra location. But we have to iterate through each UUID.
            if let Ok(entries) = read_dir(from_home_dir(".local/share/astra/game/")) {
                entries
                    .flatten()
                    .flat_map(|entry| {
                        let Ok(meta) = entry.metadata() else {
                            return vec![];
                        };
                        if meta.is_dir() {
                            return vec![entry.path()];
                        }
                        vec![]
                    })
                    .for_each(|path| {
                        let path = path.to_str().unwrap().to_string();
                        let version = read_version(&path);

                        install_dirs.push(ExistingGameDirectory {
                            install_type: ExistingInstallType::Astra,
                            path,
                            version,
                        })
                    });
            }
        }
        "macos" => {
            // Official Launcher (macOS)
            {
                let path = from_home_dir(
                    "Library/Application Support/FINAL FANTASY XIV ONLINE/Bottles/published_Final_Fantasy/drive_c/Program Files (x86)/SquareEnix/FINAL FANTASY XIV - A Realm Reborn",
                );
                install_dirs.push(ExistingGameDirectory {
                    install_type: ExistingInstallType::OfficialLauncher,
                    path: path.clone(),
                    version: read_version(&path),
                });
            }

            // TODO: add XIV on Mac
        }
        "windows" => {
            // Official install (Wine)
            {
                let path =
                    "C:\\Program Files (x86)\\SquareEnix\\FINAL FANTASY XIV - A Realm Reborn"
                        .to_string();
                install_dirs.push(ExistingGameDirectory {
                    install_type: ExistingInstallType::OfficialLauncher,
                    path: path.clone(),
                    version: read_version(&path),
                });
            }

            // TODO: Add Astra
        }
        &_ => {}
    }

    install_dirs
        .into_iter()
        .filter(|dir| is_valid_game_dir(&dir.path))
        .filter(|dir| !dir.version.is_empty())
        .collect()
}

/// An existing user directory
pub struct ExistingUserDirectory {
    /// The application where this directory was from
    pub install_type: ExistingInstallType,
    /// The path to the user folder
    pub path: String,
}

/// Finds existing user folders on disk. Will only return locations that actually have files in them, and a really basic check to see if the data is valid.
pub fn find_existing_user_dirs() -> Vec<ExistingUserDirectory> {
    let mut user_dirs = Vec::new();
    #[allow(deprecated)] // We still want std::env::home_dir
    let Some(_) = home_dir() else {
        return user_dirs;
    };

    match std::env::consts::OS {
        "linux" => {
            // Official install (Wine)
            user_dirs.push(ExistingUserDirectory {
                install_type: ExistingInstallType::OfficialLauncher,
                path: from_home_dir("Documents/My Games/FINAL FANTASY XIV - A Realm Reborn"),
            });

            // XIVLauncherCore location
            user_dirs.push(ExistingUserDirectory {
                install_type: ExistingInstallType::XIVLauncherCore,
                path: from_home_dir(".xlcore/ffxivConfig"),
            });

            // Astra location. But we have to iterate through each UUID.
            if let Ok(entries) = read_dir(from_home_dir(".local/share/astra/user/")) {
                entries
                    .flatten()
                    .flat_map(|entry| {
                        let Ok(meta) = entry.metadata() else {
                            return vec![];
                        };
                        if meta.is_dir() {
                            return vec![entry.path()];
                        }
                        vec![]
                    })
                    .for_each(|path| {
                        user_dirs.push(ExistingUserDirectory {
                            install_type: ExistingInstallType::Astra,
                            path: path.into_os_string().into_string().unwrap(),
                        })
                    });
            }
        }
        "macos" => {
            // Official install (Wine)
            user_dirs.push(ExistingUserDirectory {
                install_type: ExistingInstallType::OfficialLauncher,
                path: from_home_dir("Documents/My Games/FINAL FANTASY XIV - A Realm Reborn"),
            })

            // TODO: Add XIV on Mac?
        }
        "windows" => {
            // Official install
            user_dirs.push(ExistingUserDirectory {
                install_type: ExistingInstallType::OfficialLauncher,
                path: from_home_dir("Documents/My Games/FINAL FANTASY XIV - A Realm Reborn"),
            })

            // TODO: Add Astra
        }
        &_ => {}
    }

    user_dirs
        .into_iter()
        .filter(|dir| is_valid_user_dir(&dir.path))
        .collect()
}

fn from_home_dir(path: &'static str) -> String {
    #[allow(deprecated)] // We still want std::env::home_dir
    let mut new_path = home_dir().unwrap();
    new_path.push(path);
    new_path.into_os_string().into_string().unwrap()
}

fn is_valid_game_dir(path: &String) -> bool {
    let mut d = PathBuf::from(path);

    // Check for the dir itself
    if fs::metadata(d.as_path()).is_err() {
        return false;
    }

    // Check for "game"
    d.push("game");

    if fs::metadata(d.as_path()).is_err() {
        return false;
    }

    // Check for "boot"
    d.pop();
    d.push("boot");

    if fs::metadata(d.as_path()).is_err() {
        return false;
    }

    true
}

fn is_valid_user_dir(path: &String) -> bool {
    let mut d = PathBuf::from(path);

    // Check for the dir itself
    if fs::metadata(d.as_path()).is_err() {
        return false;
    }

    // Check for "FFXIV.cfg"
    d.push("FFXIV.cfg");

    if fs::metadata(d.as_path()).is_err() {
        return false;
    }

    true
}
