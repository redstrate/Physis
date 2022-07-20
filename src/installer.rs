use std::ffi::{CStr, CString};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::os::raw::c_char;

const FILES_TO_EXTRACT: [&str; 3] = ["data1.cab", "data1.hdr", "data2.cab"];

const BOOT_COMPONENT_FILES: [&str; 18] = [
    "cef_license.txt",
    "FFXIV.ico",
    "ffxivboot.exe",
    "ffxivboot.ver",
    "ffxivboot64.exe",
    "ffxivconfig.exe",
    "ffxivconfig64.exe",
    "ffxivlauncher.exe",
    "ffxivlauncher64.exe",
    "ffxivsysinfo.exe",
    "ffxivsysinfo64.exe",
    "ffxivupdater.exe",
    "ffxivupdater64.exe",
    "FFXIV_sysinfo.ico",
    "icudt.dll",
    "libcef.dll",
    "license.txt",
    "locales/reserved.txt"
];

const GAME_COMPONENT_FILES: [&str; 1] = ["ffxivgame.ver"];

#[repr(C)]
struct Unshield;

extern "C" {
    fn unshield_open(filename: *const c_char) -> *mut Unshield;
    fn unshield_close(unshield: *mut Unshield);

    fn unshield_set_log_level(level : i32);

    fn unshield_file_count(unshield: *mut Unshield) -> i32;
    fn unshield_file_name(unshield: *mut Unshield, index : i32) -> *const c_char;
    fn unshield_file_save(unshield: *mut Unshield, index : i32, filename: *const c_char) -> bool;
}

/// Installs the game from the provided retail installer.
pub unsafe fn install_game(installer_path : &str, game_directory : &str) {
    let installer_file = std::fs::read(installer_path).unwrap();

    let mut file_size = installer_file.len();
    let mut last_position = 0;
    let mut last_filename = "";
    for filename in FILES_TO_EXTRACT {
        let needle = format!("Disk1\\{}", filename);

        let position = installer_file.windows(needle.len()).position(|window| window == needle.as_str().as_bytes());
        if position == None {
            break;
        }

        let position = position.unwrap();

        if last_position != 0 {
            let mut new_file = File::create(last_filename).unwrap();

            if last_filename == "data1.hdr" {
                new_file.write(&installer_file[last_position + 30..position - 42]);
            } else {
                new_file.write(&installer_file[last_position + 33..position - 42]);
            }

            last_position = position;
        }

        last_position = position;
        last_filename = filename;
        file_size -= (position + 4) - last_position;
    }

    let mut new_file = File::create(last_filename).unwrap();

    new_file.write(&installer_file[last_position + 33..installer_file.len() - 42]);

    fs::create_dir_all(format!("{game_directory}/boot"));
    fs::create_dir_all(format!("{game_directory}/game"));

    unsafe {
        // set unshield to shut up
        unshield_set_log_level(0);

        let unshield = unshield_open(b"data1.cab".as_ptr() as *const c_char);
        let file_count = unshield_file_count(unshield);

        for i in 0..file_count {
            let filename = CStr::from_ptr(unshield_file_name(unshield, i)).to_string_lossy();

            for boot_name in BOOT_COMPONENT_FILES {
                if boot_name == filename {
                    let save_filename = format!("{game_directory}/boot/{boot_name}");
                    unshield_file_save(unshield, i, CString::new(save_filename).unwrap().as_ptr());
                }
            }

            for game_name in GAME_COMPONENT_FILES {
                if game_name == filename {
                    let save_filename = format!("{game_directory}/game/{game_name}");
                    unshield_file_save(unshield, i, CString::new(save_filename).unwrap().as_ptr());
                }
            }
        }

        unshield_close(unshield);
    }
}

