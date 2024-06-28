// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::ffi::{CStr, CString, NulError};
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
    "locales/reserved.txt",
];

const GAME_COMPONENT_FILES: [&str; 1] = ["ffxivgame.ver"];

#[repr(C)]
struct Unshield {
    _private: [u8; 0],
}

extern "C" {
    fn unshield_open(filename: *const c_char) -> *mut Unshield;
    fn unshield_close(unshield: *mut Unshield);

    fn unshield_set_log_level(level: i32);

    fn unshield_file_count(unshield: *mut Unshield) -> i32;
    fn unshield_file_name(unshield: *mut Unshield, index: i32) -> *const c_char;
    fn unshield_file_save(unshield: *mut Unshield, index: i32, filename: *const c_char) -> bool;
}

pub enum InstallError {
    IOFailure,
    FFIFailure,
}

impl From<std::io::Error> for InstallError {
    fn from(_: std::io::Error) -> Self {
        InstallError::IOFailure
    }
}

impl From<NulError> for InstallError {
    fn from(_: NulError) -> Self {
        InstallError::FFIFailure
    }
}

/// Installs the game from the provided retail installer.
pub fn install_game(installer_path: &str, game_directory: &str) -> Result<(), InstallError> {
    let installer_file = fs::read(installer_path).unwrap();

    let mut last_position = 0;
    let mut last_filename = "";
    for filename in FILES_TO_EXTRACT {
        let needle = format!("Disk1\\{}", filename);

        let position = installer_file
            .windows(needle.len())
            .position(|window| window == needle.as_str().as_bytes());
        if position == None {
            break;
        }

        let position = position.unwrap();

        if last_position != 0 {
            let mut temp_dir = std::env::temp_dir();
            temp_dir.push(last_filename);

            let mut new_file = File::create(temp_dir).unwrap();

            if last_filename == "data1.hdr" {
                new_file.write_all(&installer_file[last_position + 30..position - 42])?;
            } else {
                new_file.write_all(&installer_file[last_position + 33..position - 42])?;
            }
        }

        last_position = position;
        last_filename = filename;
    }

    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(last_filename);

    let mut new_file = File::create(temp_dir).unwrap();

    new_file.write_all(&installer_file[last_position + 33..installer_file.len() - 42])?;

    fs::create_dir_all(format!("{game_directory}/boot"))?;
    fs::create_dir_all(format!("{game_directory}/game"))?;

    // set unshield to shut up
    unsafe { unshield_set_log_level(0) };

    let mut temp_dir = std::env::temp_dir();
    temp_dir.push("data1.cab");
    let temp_dir_string = CString::new(temp_dir.to_str().unwrap())?;

    let unshield = unsafe { unshield_open(temp_dir_string.as_ptr()) };
    let file_count = unsafe { unshield_file_count(unshield) };

    for i in 0..file_count {
        let filename = unsafe { CStr::from_ptr(unshield_file_name(unshield, i)).to_string_lossy() };

        for boot_name in BOOT_COMPONENT_FILES {
            if boot_name == filename {
                let save_filename = format!("{game_directory}/boot/{boot_name}");
                let save_filename_c = CString::new(save_filename)?;
                unsafe { unshield_file_save(unshield, i, save_filename_c.as_ptr()) };
            }
        }

        for game_name in GAME_COMPONENT_FILES {
            if game_name == filename {
                let save_filename = format!("{game_directory}/game/{game_name}");
                let save_filename_c = CString::new(save_filename)?;
                unsafe { unshield_file_save(unshield, i, save_filename_c.as_ptr()) };
            }
        }
    }

    unsafe {
        unshield_close(unshield);
    }

    Ok(())
}
