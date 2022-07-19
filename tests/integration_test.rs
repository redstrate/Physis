use std::env;

#[test]
fn test_index_read() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    physis::index::IndexFile::from_existing(format!("{}/game/sqpack/ffxiv/000000.win32.index", game_dir).as_str());
}

#[test]
fn test_gamedata_extract() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let mut gamedata = physis::gamedata::GameData::from_existing(format!("{}/game", game_dir).as_str()).unwrap();

    gamedata.reload_repositories();

    assert!(gamedata.extract("exd/root.exl").is_some());
}