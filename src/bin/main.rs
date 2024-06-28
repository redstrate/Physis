use std::fs::write;
use physis::common::Platform;
use physis::gamedata::GameData;
use physis::mtrl::Material;

fn main() {
    let game_data = GameData::from_existing(Platform::Win32, "/home/josh/.local/share/astra/game/{2bdefc9d-3382-45d6-952f-ae9b918c4764}/game");

    let p = "bg/ffxiv/fst_f1/fld/f1f3/material/f1f3_w1_mizu4a.mtrl";
    let m_data = game_data.unwrap().extract(p).unwrap();
    write("/home/josh/test.mtrl", &m_data);
    let m = Material::from_existing(&m_data);
    println!("{:#?}", m);
}