// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
    // Windows doesn't ship pkgconfig files, typically. At least in our build system.
    #[cfg(all(not(target_os = "windows"), not(target_family = "wasm")))]
    system_deps::Config::new().probe().unwrap();
}
