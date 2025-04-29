// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use physis::common::Platform;

/// Test to see if we can find the root EXL. It exists in every version, and is a pretty safe indicator whether our SqPack reading works.
#[test]
fn test_gamedata_extract() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let mut gamedata = physis::gamedata::GameData::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    )
    .unwrap();

    assert!(gamedata.extract("exd/root.exl").is_some());
}
