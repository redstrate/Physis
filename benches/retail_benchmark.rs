// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;

use brunch::Bench;
use physis::common::Platform;

fn reload_repos() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();
    physis::gamedata::GameData::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    )
    .unwrap();
}

fn fetch_data() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();
    let mut gamedata = physis::gamedata::GameData::from_existing(
        Platform::Win32,
        format!("{}/game", game_dir).as_str(),
    )
    .unwrap();

    gamedata.extract("exd/root.exl");
}

brunch::benches!(
    Bench::new("gamedata reloading repositories").run(reload_repos),
    Bench::new("gamedata extract").run(fetch_data),
);
