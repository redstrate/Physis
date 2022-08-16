use crate::race::{get_race_id, Gender, Race, Subrace};

#[repr(u8)]
#[derive(Debug, PartialEq)]
/// The slot the item is for.
pub enum Slot {
    /// The head slot. Shorthand is "met".
    Head,
    /// The hands slot. Shorthand is "glv".
    Hands,
    /// The legs slot. Shorthand is "dwn".
    Legs,
    /// The feet slot. Shorthand is "sho".
    Feet,
    /// The body or chest slot. Shorthand is "top".
    Body,
    /// The earrings slot. Shorthand is "ear".
    Earring,
    /// The neck slot. Shorthand is "nek".
    Neck,
    /// The ring slot. Shorthand is "rir".
    Rings,
    /// The wrists slot. Shorthand is "wrs".
    Wrists,
}

/// Returns the shorthand abbreviation of `slot`. For example, Body's shorthand is "top".
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
        Slot::Wrists => "wrs",
    }
}

/// Determines the correct slot from an id. This can fail, so a None is returned when no slot matches
/// that id.
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
        _ => None,
    }
}

/// Determines the correct slot from an id. This can fail, so a None is returned when no slot matches
/// that id.
pub fn get_slot_from_abbreviation(abrev: &str) -> Option<Slot> {
    match abrev {
        "met" => Some(Slot::Head),
        "glv" => Some(Slot::Hands),
        "dwn" => Some(Slot::Legs),
        "sho" => Some(Slot::Feet),
        "top" => Some(Slot::Body),
        "ear" => Some(Slot::Earring),
        "nek" => Some(Slot::Neck),
        "rir" => Some(Slot::Rings),
        "wrs" => Some(Slot::Wrists),
        _ => None,
    }
}

/// Builds a game path to the equipment specified.
pub fn build_equipment_path(
    model_id: i32,
    race: Race,
    subrace: Subrace,
    gender: Gender,
    slot: Slot,
) -> String {
    format!(
        "chara/equipment/e{:04}/model/c{:04}e{:04}_{}.mdl",
        model_id,
        get_race_id(race, subrace, gender).unwrap(),
        model_id,
        get_slot_abbreviation(slot)
    )
}

pub fn deconstruct_equipment_path(path: &str) -> Option<(i32, Slot)> {
    let model_id = &path[6..10];
    let slot_name = &path[11..14];

    Some((
        model_id.parse().ok()?,
        get_slot_from_abbreviation(slot_name)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equipment_path() {
        assert_eq!(
            build_equipment_path(0, Race::Hyur, Subrace::Midlander, Gender::Male, Slot::Body),
            "chara/equipment/e0000/model/c0101e0000_top.mdl"
        );
    }
}
