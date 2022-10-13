use std::collections::HashMap;
use physis::index;
use std::env;
use std::process::Command;
use physis::installer::install_game;
use physis::patch::apply_patch;
use walkdir::{DirEntry, WalkDir};

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

    let mut gamedata =
        physis::gamedata::GameData::from_existing(format!("{}/game", game_dir).as_str()).unwrap();

    gamedata.reload_repositories();

    assert!(gamedata.extract("exd/root.exl").is_some());
}

#[test]
#[cfg_attr(not(feature = "patch_testing"), ignore)]
fn test_patching() {
    let patch_dir = env::var("FFXIV_PATCH_DIR").unwrap();
    let patcher_exe = env::var("FFXIV_XIV_LAUNCHER_PATCHER").unwrap();
    let installer_exe = env::var("FFXIV_INSTALLER").unwrap();

    println!("Beginning game installation...");

    // FIXME: lmao what is going on here
    let mut game_dir = env::temp_dir();
    game_dir.push("game_test");

    if std::fs::read_dir(&game_dir).ok().is_some() {
        std::fs::remove_dir_all(&game_dir).unwrap();
    }

    std::fs::create_dir_all(&game_dir).unwrap();

    install_game(&installer_exe, game_dir.as_path().to_str().unwrap()).ok().unwrap();

    let mut xivlauncher_game_dir = env::temp_dir();
    xivlauncher_game_dir.push("game_test_xivlauncher");

    if std::fs::read_dir(&xivlauncher_game_dir).ok().is_some() {
        std::fs::remove_dir_all(&xivlauncher_game_dir).unwrap();
    }
    std::fs::create_dir_all(&xivlauncher_game_dir).unwrap();

    install_game(&installer_exe, xivlauncher_game_dir.as_path().to_str().unwrap()).ok().unwrap();

    let xivlauncher_game_dir = xivlauncher_game_dir.as_path();
    let physis_game_dir = game_dir.as_path();

    // TODO: only the first two patches are checked
    let patches = ["game/D2017.07.11.0000.0001.patch",
        "game/D2017.09.24.0000.0001.patch"];

    println!("The game installation is now complete. Now testing game patching...");

    // run it on xiv's patchinstaller first
    // TODO: check for windows systems
    for patch in patches {
        let patch_path = format!("Z:\\{}\\{}", patch_dir, &patch);
        let game_dir = format!("Z:\\{}\\game", xivlauncher_game_dir.to_str().unwrap());

        let output = Command::new("/usr/bin/wine")
            .args([&patcher_exe,
            "install",
            &patch_path,
            &game_dir])
            .output()
            .unwrap();

        println!("{:#?}", output);
    }

    // now run it on physis
    for patch in patches {
        let patch_path = format!("{}/{}", patch_dir, &patch);
        let data_dir = format!("{}/{}", physis_game_dir.to_str().unwrap(), "game");

        apply_patch(&data_dir, &patch_path).unwrap();
    }

    println!("Game patching is now complete. Proceeding to checksum matching...");

    let mut xivlauncher_files : HashMap<String, [u8; 64]> = HashMap::new();

    println!("{:#?}", xivlauncher_game_dir);

    // TODO: consolidate into a closure or generic function
    WalkDir::new(xivlauncher_game_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .for_each(|x| {
            let file = std::fs::read(x.path()).unwrap();

            let mut hash = hmac_sha512::Hash::new();
            hash.update(&file);
            let sha = hash.finalize();

            let mut rel_path = x.path();
            rel_path = rel_path.strip_prefix(xivlauncher_game_dir).unwrap();

            xivlauncher_files.insert(rel_path.to_str().unwrap().to_string(), sha);
        });

    let mut physis_files : HashMap<String, [u8; 64]> = HashMap::new();

    WalkDir::new(physis_game_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .for_each(|x| {
            if !x.file_type().is_dir() {
                let file = std::fs::read(x.path()).unwrap();

                let mut hash = hmac_sha512::Hash::new();
                hash.update(&file);
                let sha = hash.finalize();

                let mut rel_path = x.path();
                rel_path = rel_path.strip_prefix(physis_game_dir).unwrap();

                physis_files.insert(rel_path.to_str().unwrap().to_string(), sha);
            }
        });

    assert_eq!(physis_files, xivlauncher_files);
}