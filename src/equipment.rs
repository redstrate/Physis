use crate::race::{Gender, get_race_id, Race, Subrace};

#[repr(u8)]
pub enum Slot {
    Head,
    Hands,
    Legs,
    Feet,
    Body,
    Earring,
    Neck,
    Rings,
    Wrists,
}

pub fn get_slot_abbreviation(slot: Slot) -> &'static str {
    match slot {
        Slot::Head => "met",
        Slot::Hands => "glv",
        Slot::Legs => "dwn",
        Slot::Feet => "sho",
        Slot::Body => "top",
        Slot::Earring => "ear",
        Slot::Neck => "nek",
        Slot::Rings => "rir",
        Slot::Wrists => "wrs"
    }
}

pub fn get_slot_from_id(id: i32) -> Option<Slot> {
    match id {
        3 => Some(Slot::Head),
        5 => Some(Slot::Hands),
        7 => Some(Slot::Legs),
        8 => Some(Slot::Feet),
        4 => Some(Slot::Body),
        9 => Some(Slot::Earring),
        10 => Some(Slot::Neck),
        12 => Some(Slot::Rings),
        11 => Some(Slot::Wrists),
        _ => None
    }
}

pub fn build_equipment_path(model_id: i32, race: Race, subrace: Subrace, gender: Gender, slot: Slot) -> String {
    format!("chara/equipment/e{:04}/model/c{:04}e{:04}_{}.mdl",
            model_id,
            get_race_id(race, subrace, gender).unwrap(),
            model_id,
            get_slot_abbreviation(slot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equipment_path() {
        assert_eq!(build_equipment_path(0, Race::Hyur, Subrace::Midlander, Gender::Male, Slot::Body), "chara/equipment/e0000/model/c0101e0000_top.mdl");
    }
}