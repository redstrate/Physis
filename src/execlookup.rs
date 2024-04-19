// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;

fn from_u16(from: &mut [u16]) -> &[u8] {
    #[cfg(target_endian = "little")]
    from.iter_mut().for_each(|word| *word = word.to_be());

    let ptr: *const u8 = from.as_ptr().cast();
    let len = from.len().checked_mul(2).unwrap();

    unsafe { std::slice::from_raw_parts(ptr, len) }
}

fn find_needle(installer_file: &Vec<u8>, needle: &str) -> Option<String> {
    let mut needle: Vec<u16> = needle.encode_utf16().collect();
    let bytes = from_u16(&mut needle);

    let Some(mut position) = installer_file
        .windows(bytes.len())
        .position(|window| window == bytes) else {
        return None;
    };
    
    let parse_char_at_position = |position: usize| {
        let upper = installer_file[position];
        let lower = installer_file[position + 1];

        let result = char::decode_utf16([((upper as u16) << 8) | lower as u16])
            .map(|r| r.map_err(|e| e.unpaired_surrogate()))
            .collect::<Vec<_>>();

        result[0]
    };

    let mut string: String = String::new();

    let mut last_char = parse_char_at_position(position);
    while last_char.is_ok() && last_char.unwrap() != '\0' {
        string.push(last_char.unwrap());

        position += 2;
        last_char = parse_char_at_position(position);
    }

    Some(string)
}

/// Extract the frontier URL from ffxivlauncher.exe
pub fn extract_frontier_url(launcher_path: &str) -> Option<String> {
    let installer_file = fs::read(launcher_path).unwrap();

    // Old Frontier URL format
    if let Some(url) = find_needle(&installer_file, "https://frontier.ffxiv.com") {
        return Some(url);
    }

    // New Frontier URL format
    if let Some(url) = find_needle(&installer_file, "https://launcher.finalfantasyxiv.com") {
        return Some(url);
    }

    None
}