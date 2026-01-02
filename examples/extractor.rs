// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use physis::common::Platform;
use physis::resource::{Resource, SqPackRelease, SqPackResource};
use std::env;
use std::fs::File;
use std::io::Write;

/// A simple program that allows a user to extract raw files from the game
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Usage: extractor [game dir] [filepath_to_extract] [destination]");
        return;
    }

    // Collect our arguments
    let game_dir = &args[1];
    let file_path = &args[2];
    let destination_path = &args[3];

    // Create a GameData struct, this manages the repositories. It allows us to easily extract files.
    let mut game_data = SqPackResource::from_existing(game_dir);

    // Extract said file:
    let Some(game_file) = game_data.read(file_path) else {
        println!("File {} not found!", file_path);
        return;
    };

    // Create the file to write into.
    let Ok(mut file) = File::create(destination_path) else {
        println!("Failed to open file {} for writing.", destination_path);
        return;
    };

    // Since GameData::extract returns a byte buffer, it's trivial to write that to a file on disk.
    if file.write_all(&game_file).is_err() {
        println!("Failed to write to file {}.", destination_path);
        return;
    };

    println!(
        "Successfully extracted {} to {}!",
        file_path, destination_path
    );
}
