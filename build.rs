// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
    #[cfg(feature = "game_install")]
    println!("cargo::rustc-link-lib=unshield");
}
