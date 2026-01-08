// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::race::{Gender, Race, Tribe, get_race_id};

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// The slot the item is for.
pub enum Slot {
    /// No applicable slot.
    // NOTE: this invariant isn't great for Rust, but is needed for the C api.
    Invalid,
    /// The main weapon slot.
    MainHand,
    /// The off-hand weapon slot.
    OffHand,
    /// The head slot.
    Head,
    /// The hands slot.
    Hands,
    /// The legs slot.
    Legs,
    /// The feet slot.
    Feet,
    /// The body or chest slot.
    Body,
    /// The earrings slot.
    Earring,
    /// The neck slot.
    Neck,
    /// The wrists slot.
    Wrists,
    /// The right ring slot.
    RingLeft,
    /// The left ring slot.
    RingRight,
}

/// Returns the shorthand abbreviation of `slot`. For example, Body's shorthand is "top".
pub fn get_slot_abbreviation(slot: Slot) -> Option<&'static str> {
    match slot {
        Slot::Head => Some("met"),
        Slot::Hands => Some("glv"),
        Slot::Legs => Some("dwn"),
        Slot::Feet => Some("sho"),
        Slot::Body => Some("top"),
        Slot::Earring => Some("ear"),
        Slot::Neck => Some("nek"),
        Slot::Wrists => Some("wrs"),
        Slot::RingLeft => Some("ril"),
        Slot::RingRight => Some("rir"),
        _ => None,
    }
}

/// Determines the correct slot from an id. This can fail, so `Invalid` is returned when no slot matches
/// that id.
pub fn get_slot_from_id(id: i32) -> Slot {
    match id {
        1 => Slot::MainHand,
        2 => Slot::OffHand,
        3 => Slot::Head,
        4 => Slot::Body,
        5 => Slot::Hands,
        7 => Slot::Legs,
        8 => Slot::Feet,
        9 => Slot::Earring,
        10 => Slot::Neck,
        11 => Slot::Wrists,
        12 => Slot::RingLeft,
        13 => Slot::RingRight,
        _ => Slot::Invalid,
    }
}

/// Determines the correct slot from an id. This can fail, so `Invalid` is returned when no slot matches
/// that id.
pub fn get_slot_from_abbreviation(abrev: &str) -> Slot {
    match abrev {
        "met" => Slot::Head,
        "glv" => Slot::Hands,
        "dwn" => Slot::Legs,
        "sho" => Slot::Feet,
        "top" => Slot::Body,
        "ear" => Slot::Earring,
        "nek" => Slot::Neck,
        "wrs" => Slot::Wrists,
        "ril" => Slot::RingLeft,
        "rir" => Slot::RingRight,
        _ => Slot::Invalid,
    }
}

/// Builds a game path to the equipment specified.
pub fn build_equipment_path(
    model_id: i32,
    race: Race,
    tribe: Tribe,
    gender: Gender,
    slot: Slot,
) -> String {
    let race_id = get_race_id(race, tribe, gender).unwrap();
    match slot {
        Slot::MainHand | Slot::OffHand => {
            format!(
                "chara/weapon/w{model_id:04}/obj/body/b{race_id:04}/model/w{model_id:04}b{race_id:04}.mdl"
            )
        }
        Slot::Neck | Slot::Earring | Slot::Wrists | Slot::RingLeft | Slot::RingRight => {
            format!(
                "chara/accessory/a{model_id:04}/model/c{race_id:04}a{model_id:04}_{}.mdl",
                get_slot_abbreviation(slot).unwrap_or_default()
            )
        }
        _ => {
            format!(
                "chara/equipment/e{model_id:04}/model/c{race_id:04}e{model_id:04}_{}.mdl",
                get_slot_abbreviation(slot).unwrap_or_default()
            )
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum CharacterCategory {
    Body,
    Hair,
    Face,
    Tail,
    Ear,
}

pub fn get_character_category_path(category: CharacterCategory) -> &'static str {
    match category {
        CharacterCategory::Body => "body",
        CharacterCategory::Hair => "hair",
        CharacterCategory::Face => "face",
        CharacterCategory::Tail => "tail",
        CharacterCategory::Ear => "zear",
    }
}

pub fn get_character_category_abbreviation(category: CharacterCategory) -> &'static str {
    match category {
        CharacterCategory::Body => "top",
        CharacterCategory::Hair => "hir",
        CharacterCategory::Face => "fac",
        CharacterCategory::Tail => "til",
        CharacterCategory::Ear => "zer",
    }
}

pub fn get_character_category_prefix(category: CharacterCategory) -> &'static str {
    match category {
        CharacterCategory::Body => "b",
        CharacterCategory::Hair => "h",
        CharacterCategory::Face => "f",
        CharacterCategory::Tail => "t",
        CharacterCategory::Ear => "z",
    }
}

/// Builds a game path to the equipment specified.
pub fn build_character_path(
    category: CharacterCategory,
    body_ver: i32,
    race: Race,
    tribe: Tribe,
    gender: Gender,
) -> String {
    let category_path = get_character_category_path(category);
    let race_id = get_race_id(race, tribe, gender).unwrap();
    let category_abbreviation = get_character_category_abbreviation(category);
    let category_prefix = get_character_category_prefix(category);
    format!(
        "chara/human/c{race_id:04}/obj/{category_path}/{category_prefix}{body_ver:04}/model/c{race_id:04}{category_prefix}{body_ver:04}_{category_abbreviation}.mdl"
    )
}

/// Builds a material path for a specific gear
pub fn build_gear_material_path(gear_id: i32, gear_version: i32, material_name: &str) -> String {
    format!("chara/equipment/e{gear_id:04}/material/v{gear_version:04}{material_name}")
}

/// Builds a skin material path for a character
pub fn build_skin_material_path(race_code: i32, body_code: i32, material_name: &str) -> String {
    format!("chara/human/c{race_code:04}/obj/body/b{body_code:04}/material/v0001{material_name}")
}

/// Builds a face material path for a character
pub fn build_face_material_path(race_code: i32, face_code: i32, material_name: &str) -> String {
    format!("chara/human/c{race_code:04}/obj/face/f{face_code:04}/material{material_name}")
}

/// Builds a hair material path for a character
pub fn build_hair_material_path(race_code: i32, hair_code: i32, material_name: &str) -> String {
    format!("chara/human/c{race_code:04}/obj/hair/h{hair_code:04}/material/v0001{material_name}")
}

/// Builds a ear material path for a character
pub fn build_ear_material_path(race_code: i32, ear_code: i32, material_name: &str) -> String {
    format!("chara/human/c{race_code:04}/obj/ear/e{ear_code:04}/material/v0001{material_name}")
}

/// Builds a tail material path for a character
pub fn build_tail_material_path(race_code: i32, tail_code: i32, material_name: &str) -> String {
    format!("chara/human/c{race_code:04}/obj/tail/t{tail_code:04}/material/v0001{material_name}")
}

pub fn deconstruct_equipment_path(path: &str) -> Option<(i32, Slot)> {
    let model_id = &path[6..10];
    let slot_name = &path[11..14];

    Some((
        model_id.parse().ok()?,
        get_slot_from_abbreviation(slot_name),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equipment_path() {
        assert_eq!(
            build_equipment_path(0, Race::Hyur, Tribe::Midlander, Gender::Male, Slot::Body),
            "chara/equipment/e0000/model/c0101e0000_top.mdl"
        );
    }

    #[test]
    fn test_deconstruct() {
        assert_eq!(
            deconstruct_equipment_path("c0101e0000_top.mdl"),
            Some((0, Slot::Body))
        );
    }
}
