use criterion::{criterion_group, criterion_main, Criterion};
use physis::sqpack::calculate_hash;
use std::env;

fn reload_repos() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();
    let mut gamedata =
        physis::gamedata::GameData::from_existing(format!("{}/game", game_dir).as_str()).unwrap();

    gamedata.reload_repositories();
}

fn bench_calculate_hash() {
    calculate_hash("exd/root.exl");
}

fn fetch_data() {
    let game_dir = env::var("FFXIV_GAME_DIR").unwrap();
    let mut gamedata =
        physis::gamedata::GameData::from_existing(format!("{}/game", game_dir).as_str()).unwrap();

    gamedata.reload_repositories();

    gamedata.extract("exd/root.exl");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("hash calc", |b| b.iter(bench_calculate_hash));
    c.bench_function("gamedata reloading repositories", |b| b.iter(reload_repos));
    c.bench_function("gamedata extract", |b| b.iter(fetch_data));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
