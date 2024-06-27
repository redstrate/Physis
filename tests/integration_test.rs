// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use hmac_sha512::Hash;
use physis::patch::apply_patch;
use std::env;
use std::fs::{read, read_dir};
use std::process::Command;

use physis::common::Platform;
use physis::fiin::FileInfo;
use physis::index;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[test]
#[cfg_attr(not(feature = "retail_game_testing"), ignore)]
fn test_index_read() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    index::IndexFile::from_existing(
        format!("{}/game/sqpack/ffxiv/000000.win32.index", game_dir).as_str(),
    );
}

#[test]
#[cfg_attr(not(feature = "retail_game_testing"), ignore)]
fn test_gamedata_extract() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let mut gamedata = physis::gamedata::GameData::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    )
    .unwrap();

    assert!(gamedata.extract("exd/root.exl").is_some());
}

#[test]
#[cfg_attr(not(feature = "retail_game_testing"), ignore)]
fn test_fiin() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let fiin_path = format!("{game_dir}/boot/fileinfo.fiin");
    let fiin = FileInfo::from_existing(&read(fiin_path).unwrap()).unwrap();

    assert_eq!(fiin.entries[0].file_name, "steam_api.dll");
    assert_eq!(fiin.entries[1].file_name, "steam_api64.dll");
}
