// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use brunch::Bench;
use physis::index::IndexFile;

fn bench_calculate_hash() {
    IndexFile::calculate_hash("exd/root.exl");
}

brunch::benches!(Bench::new("hash c alc").run(bench_calculate_hash),);
