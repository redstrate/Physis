// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

fn main() {
    // Windows doesn't ship pkgconfig files, typically. At least in our build system.
    #[cfg(not(target_os = "windows"))]
    system_deps::Config::new().probe().unwrap();
}
