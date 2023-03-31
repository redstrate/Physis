use physis::fiin::FileInfo;
use physis::index;
use std::env;
use std::fs::read;

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
#[cfg_attr(not(feature = "retail_game_testing"), ignore)]
fn test_fiin() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();

    let fiin_path = format!("{game_dir}/boot/fileinfo.fiin");
    let fiin = FileInfo::from_existing(&read(fiin_path).unwrap()).unwrap();

    assert_eq!(fiin.entries[0].file_name, "steam_api.dll");
    assert_eq!(fiin.entries[1].file_name, "steam_api64.dll");
}

#[cfg(feature = "patch_testing")]
fn make_temp_install_dir(name: &str) -> String {
    use physis::installer::install_game;

    let installer_exe = env::var("FFXIV_INSTALLER").unwrap();

    let mut game_dir = env::home_dir().unwrap();
    game_dir.push(name);

    if std::fs::read_dir(&game_dir).ok().is_some() {
        std::fs::remove_dir_all(&game_dir).unwrap();
    }

    std::fs::create_dir_all(&game_dir).unwrap();

    install_game(&installer_exe, game_dir.as_path().to_str().unwrap())
        .ok()
        .unwrap();

    game_dir.as_path().to_str().unwrap().parse().unwrap()
}

#[cfg(feature = "patch_testing")]
fn fill_dir_hash(game_dir: &str) -> HashMap<String, [u8; 64]> {
    let mut file_hashes: HashMap<String, [u8; 64]> = HashMap::new();

    WalkDir::new(game_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .for_each(|x| {
            let file = std::fs::read(x.path()).unwrap();

            let mut hash = Hash::new();
            hash.update(&file);
            let sha = hash.finalize();

            let mut rel_path = x.path();
            rel_path = rel_path.strip_prefix(game_dir).unwrap();

            file_hashes.insert(rel_path.to_str().unwrap().to_string(), sha);
        });

    file_hashes
}

#[cfg(feature = "patch_testing")]
fn physis_install_patch(game_directory: &str, data_directory: &str, patch_name: &str) {
    let patch_dir = env::var("FFXIV_PATCH_DIR").unwrap();

    let patch_path = format!("{}/{}", patch_dir, &patch_name);
    let data_dir = format!("{}/{}", game_directory, data_directory);

    apply_patch(&data_dir, &patch_path).unwrap();
}

#[cfg(feature = "patch_testing")]
fn xivlauncher_install_patch(game_directory: &str, data_directory: &str, patch_name: &str) {
    let patch_dir = env::var("FFXIV_PATCH_DIR").unwrap();
    let patcher_exe = env::var("FFXIV_XIV_LAUNCHER_PATCHER").unwrap();

    let patch_path = format!("Z:\\{}\\{}", patch_dir, &patch_name);
    let game_dir = format!("Z:\\{}\\{}", game_directory, data_directory);

    // TODO: check for windows systems
    Command::new("/usr/bin/wine")
        .args([&patcher_exe, "install", &patch_path, &game_dir])
        .output()
        .unwrap();
}

#[test]
#[cfg(feature = "patch_testing")]
fn test_patching() {
    println!("Beginning game installation...");

    let physis_dir = make_temp_install_dir("game_test");
    let xivlauncher_dir = make_temp_install_dir("game_test_xivlauncher");

    let boot_patches = [
        "boot/2022.03.25.0000.0001.patch",
        "boot/2022.08.05.0000.0001.patch",
    ];

    println!("The game installation is now complete. Now running boot patching...");
    for patch in boot_patches {
        xivlauncher_install_patch(&xivlauncher_dir, "boot", patch);
        physis_install_patch(&physis_dir, "boot", patch);
    }

    let game_patches = [
        "game/H2017.06.06.0000.0001a.patch",
        "game/H2017.06.06.0000.0001b.patch",
        "game/H2017.06.06.0000.0001c.patch",
        "game/H2017.06.06.0000.0001d.patch",
        "game/H2017.06.06.0000.0001e.patch",
        "game/H2017.06.06.0000.0001f.patch",
        "game/H2017.06.06.0000.0001g.patch",
        "game/H2017.06.06.0000.0001h.patch",
        "game/H2017.06.06.0000.0001i.patch",
        "game/H2017.06.06.0000.0001j.patch",
        "game/H2017.06.06.0000.0001k.patch",
        "game/H2017.06.06.0000.0001l.patch",
        "game/H2017.06.06.0000.0001m.patch",
        "game/H2017.06.06.0000.0001n.patch",
        "game/D2017.07.11.0000.0001.patch",
        "game/D2017.09.24.0000.0001.patch",
        "ex1/H2017.06.01.0000.0001a.patch",
        "ex1/H2017.06.01.0000.0001b.patch",
        "ex1/H2017.06.01.0000.0001c.patch",
        "ex1/H2017.06.01.0000.0001d.patch",
    ];

    println!("Boot patching is now complete. Now running game patching...");

    for patch in game_patches {
        println!("Installing {}...", patch);

        xivlauncher_install_patch(&xivlauncher_dir, "game", patch);
        physis_install_patch(&physis_dir, "game", patch);
    }

    println!("Game patching is now complete. Proceeding to checksum matching...");

    let xivlauncher_files = fill_dir_hash(&xivlauncher_dir);
    let physis_files = fill_dir_hash(&physis_dir);

    for file in xivlauncher_files.keys() {
        if xivlauncher_files[file] != physis_files[file] {
            println!("{} does not match!", file);
        }
    }

    assert_eq!(physis_files, xivlauncher_files);
}
