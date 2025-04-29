// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::read_dir;

use std::path::{Path, PathBuf};

fn make_temp_install_dir(name: &str) -> String {
    use physis::installer::install_game;

    let installer_exe = env::var("FFXIV_INSTALLER")
        .expect("$FFXIV_INSTALLER needs to point to the retail installer");

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

// Shamelessly taken from https://stackoverflow.com/a/76820878
fn recurse(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let Ok(entries) = read_dir(path) else {
        return vec![];
    };
    entries
        .flatten()
        .flat_map(|entry| {
            let Ok(meta) = entry.metadata() else {
                return vec![];
            };
            if meta.is_dir() {
                return recurse(entry.path());
            }
            if meta.is_file() {
                return vec![entry.path()];
            }
            vec![]
        })
        .collect()
}

fn fill_dir_hash(game_dir: &str) -> HashMap<String, [u8; 64]> {
    let mut file_hashes: HashMap<String, [u8; 64]> = HashMap::new();

    recurse(game_dir).into_iter().for_each(|x| {
        let path = x.as_path();
        let file = std::fs::read(path).unwrap();

        let mut hash = Hash::new();
        hash.update(&file);
        let sha = hash.finalize();

        let mut rel_path = path;
        rel_path = rel_path.strip_prefix(game_dir).unwrap();

        file_hashes.insert(rel_path.to_str().unwrap().to_string(), sha);
    });

    file_hashes
}

fn physis_install_patch(game_directory: &str, data_directory: &str, patch_name: &str) {
    let patch_dir = env::var("FFXIV_PATCH_DIR").unwrap();

    let patch_path = format!("{}/{}", patch_dir, &patch_name);
    let data_dir = format!("{}/{}", game_directory, data_directory);

    ZiPatch::apply(&data_dir, &patch_path).unwrap();
}

fn xivlauncher_install_patch(game_directory: &str, data_directory: &str, patch_name: &str) {
    let patch_dir = env::var("FFXIV_PATCH_DIR").unwrap();
    let patcher_exe = env::var("FFXIV_XIV_LAUNCHER_PATCHER")
        .expect("$FFXIV_XIV_LAUNCHER_PATCHER must point to XIVLauncher.PatchInstaller.exe");

    let patch_path = format!("Z:\\{}\\{}", patch_dir, &patch_name);
    let game_dir = format!("Z:\\{}\\{}", game_directory, data_directory);

    // TODO: check for windows systems
    let output = Command::new("wine")
        .args([&patcher_exe, "install", &patch_path, &game_dir])
        .output()
        .unwrap();

    // If there is some kind of catostrophic failure, make sure it's printed.
    // For example, missing .NET in your wine prefix
    if (!output.status.success()) {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
    }

    assert!(output.status.success());
}

fn check_if_files_match(xivlauncher_dir: &str, physis_dir: &str) {
    let xivlauncher_files = fill_dir_hash(xivlauncher_dir);
    let physis_files = fill_dir_hash(physis_dir);

    for file in xivlauncher_files.keys() {
        if xivlauncher_files[file] != physis_files[file] {
            println!("!! {} does not match!", file);
        }
    }

    assert_eq!(physis_files, xivlauncher_files);
}

#[test]
fn test_patching() {
    println!("Beginning game installation...");

    let physis_dir = make_temp_install_dir("game_install_physis");
    let xivlauncher_dir = make_temp_install_dir("game_install_xivquicklauncher");

    println!("Done with game installation! Now checking if the checksums match first...");

    check_if_files_match(&xivlauncher_dir, &physis_dir);

    println!("* Directories match.");

    let boot_patches = [
        "boot/2023.04.28.0000.0001.patch",
        "boot/2023.04.28.0000.0001.patch",
        "boot/2024.03.07.0000.0001.patch",
        "boot/2024.03.21.0000.0001.patch",
        "boot/2024.04.09.0000.0001.patch",
        "boot/2024.05.24.0000.0001.patch",
    ];

    println!("Now beginning boot patching...");

    for patch in boot_patches {
        let patch_dir = env::var("FFXIV_PATCH_DIR")
            .expect("$FFXIV_PATCH_DIR must point to the directory where the patches are stored");
        if !Path::new(&(patch_dir + "/" + patch)).exists() {
            println!("Skipping {} because it doesn't exist locally.", patch);
            continue;
        }

        println!("Installing {}...", patch);

        xivlauncher_install_patch(&xivlauncher_dir, "boot", patch);
        physis_install_patch(&physis_dir, "boot", patch);

        check_if_files_match(&xivlauncher_dir, &physis_dir);
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
        let patch_dir = env::var("FFXIV_PATCH_DIR").unwrap();
        if !Path::new(&(patch_dir + "/" + patch)).exists() {
            println!("Skipping {} because it doesn't exist locally.", patch);
            continue;
        }

        println!("Installing {}...", patch);

        xivlauncher_install_patch(&xivlauncher_dir, "game", patch);
        physis_install_patch(&physis_dir, "game", patch);

        check_if_files_match(&xivlauncher_dir, &physis_dir);
    }

    println!("Game patching is now complete. Proceeding to checksum matching...");

    check_if_files_match(&xivlauncher_dir, &physis_dir);
}
