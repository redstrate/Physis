// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use strum_macros::{EnumCount, EnumIter, FromRepr};

/// Slot names for equipment, such as weapons and armor.
#[repr(u16)]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, FromRepr, EnumIter, EnumCount)]
pub enum EquipSlot {
    /// The main hand slot.
    MainHand = 0,
    /// The secondary/off-hand hand slot.
    OffHand,
    /// The head slot.
    Head,
    /// The body slot.
    Body,
    /// The hands slot.
    Hands,
    /// What used to be the belt slot, but it still takes space.
    Waist,
    /// The legs slot.
    Legs,
    /// The feet slot.
    Feet,
    /// The wrists/bracelets slot.
    Wrists,
    /// The neck/necklace slot.
    Neck,
    /// The ears/earrings slot.
    Ears,
    /// The right ring slot.
    RightRing,
    /// The left ring slot.
    LeftRing,
    /// The soul crystal slot.
    SoulCrystal,
}

/// Corresponds to rows in the EquipSlotCategory Excel sheet.
#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, FromRepr)]
pub enum EquipSlotCategory {
    /// No applicable slot.
    // NOTE: this invariant isn't great for Rust, but is needed for the C api.
    Invalid,
    /// The main weapon slot.
    MainHand,
    /// The off-hand weapon slot.
    OffHand,
    /// The head slot.
    Head,
    /// The body or chest slot.
    Body,
    /// The hands slot.
    Hands,
    /// What used to be the belt slot, but it still takes space.
    Waist,
    /// The legs slot.
    Legs,
    /// The feet slot.
    Feet,
    /// The earrings slot.
    Earring,
    /// The neck slot.
    Neck,
    /// The wrists slot.
    Wrists,
    /// The right ring slot.
    LeftRing,
    /// The left ring slot.
    RightRing,
}

impl EquipSlotCategory {
    /// Returns the shorthand abbreviation of `slot`. For example, Body's shorthand is "top".
    pub fn abbreviation(&self) -> Option<&'static str> {
        match self {
            EquipSlotCategory::Head => Some("met"),
            EquipSlotCategory::Hands => Some("glv"),
            EquipSlotCategory::Legs => Some("dwn"),
            EquipSlotCategory::Feet => Some("sho"),
            EquipSlotCategory::Body => Some("top"),
            EquipSlotCategory::Earring => Some("ear"),
            EquipSlotCategory::Neck => Some("nek"),
            EquipSlotCategory::Wrists => Some("wrs"),
            EquipSlotCategory::LeftRing => Some("ril"),
            EquipSlotCategory::RightRing => Some("rir"),
            _ => None,
        }
    }
}

impl From<&str> for EquipSlotCategory {
    fn from(value: &str) -> Self {
        match value {
            "met" => EquipSlotCategory::Head,
            "glv" => EquipSlotCategory::Hands,
            "dwn" => EquipSlotCategory::Legs,
            "sho" => EquipSlotCategory::Feet,
            "top" => EquipSlotCategory::Body,
            "ear" => EquipSlotCategory::Earring,
            "nek" => EquipSlotCategory::Neck,
            "wrs" => EquipSlotCategory::Wrists,
            "ril" => EquipSlotCategory::LeftRing,
            "rir" => EquipSlotCategory::RightRing,
            _ => EquipSlotCategory::Invalid,
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

impl CharacterCategory {
    pub fn path(&self) -> &'static str {
        match self {
            CharacterCategory::Body => "body",
            CharacterCategory::Hair => "hair",
            CharacterCategory::Face => "face",
            CharacterCategory::Tail => "tail",
            CharacterCategory::Ear => "zear",
        }
    }

    pub fn abbreviation(&self) -> &'static str {
        match self {
            CharacterCategory::Body => "top",
            CharacterCategory::Hair => "hir",
            CharacterCategory::Face => "fac",
            CharacterCategory::Tail => "til",
            CharacterCategory::Ear => "zer",
        }
    }

    pub fn prefix(&self) -> &'static str {
        match self {
            CharacterCategory::Body => "b",
            CharacterCategory::Hair => "h",
            CharacterCategory::Face => "f",
            CharacterCategory::Tail => "t",
            CharacterCategory::Ear => "z",
        }
    }
}

pub fn deconstruct_equipment_path(path: &str) -> Option<(i32, EquipSlotCategory)> {
    let model_id = &path[6..10];
    let slot_name = &path[11..14];

    Some((model_id.parse().ok()?, slot_name.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deconstruct() {
        assert_eq!(
            deconstruct_equipment_path("c0101e0000_top.mdl"),
            Some((0, EquipSlotCategory::Body))
        );
    }
}
