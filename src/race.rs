// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
/// Gender of the character.
pub enum Gender {
    Male = 0,
    Female = 1,
}

#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
/// The race's "tribe". Each race has two tribes, which are usually very similar (even down to the ids!)
/// with the exception of Hyurs, which have two very distinct tribes.
pub enum Tribe {
    Midlander = 1,
    Highlander = 2,
    Wildwood = 3,
    Duskwight = 4,
    Plainsfolk = 5,
    Dunesfolk = 6,
    Seeker = 7,
    Keeper = 8,
    SeaWolf = 9,
    Hellsguard = 10,
    Raen = 11,
    Xaela = 12,
    Hellion = 13,
    Lost = 14,
    Rava = 15,
    Veena = 16,
}

#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
/// The major races of Eorzea.
pub enum Race {
    Hyur = 1,
    Elezen = 2,
    Lalafell = 3,
    Miqote = 4,
    Roegadyn = 5,
    AuRa = 6,
    Hrothgar = 7,
    Viera = 8,
}

/// Gets a proper race identifier (such as 101, for Hyur-Midlander-Males) given a race, a tribe,
/// and a gender.
pub fn get_race_id(race: Race, tribe: Tribe, gender: Gender) -> Option<i32> {
    // TODO: should we check for invalid tribes like the Hyur branch does?
    match race {
        Race::Hyur => match tribe {
            Tribe::Midlander => match gender {
                Gender::Male => Some(101),
                Gender::Female => Some(201),
            },
            Tribe::Highlander => match gender {
                Gender::Male => Some(301),
                Gender::Female => Some(401),
            },
            _ => None,
        },
        Race::Elezen => match gender {
            Gender::Male => Some(501),
            Gender::Female => Some(601),
        },
        Race::Lalafell => match gender {
            Gender::Male => Some(501),
            Gender::Female => Some(601),
        },
        Race::Miqote => match gender {
            Gender::Male => Some(701),
            Gender::Female => Some(801),
        },
        Race::Roegadyn => match gender {
            Gender::Male => Some(901),
            Gender::Female => Some(1001),
        },
        Race::AuRa => match gender {
            Gender::Male => Some(1301),
            Gender::Female => Some(1401),
        },
        Race::Hrothgar => {
            match gender {
                Gender::Male => Some(1501),
                Gender::Female => Some(1601), // TODO: is this accurate as of dawntrail?
            }
        }
        Race::Viera => match gender {
            Gender::Male => Some(1701),
            Gender::Female => Some(1801),
        },
    }
}

/// Builds the path to the skeleton (sklb) file for a given `race`, `tribe` and `gender`.
pub fn build_skeleton_path(race: Race, tribe: Tribe, gender: Gender) -> String {
    format!(
        "chara/human/c{0:04}/skeleton/base/b0001/skl_c{0:04}b0001.sklb",
        get_race_id(race, tribe, gender).unwrap()
    )
}

/// Returns the two tribes associated with a given `race`. For example, `Hyur` would return `[Midlander, Highlander]`.
pub fn get_supported_tribes(race: Race) -> [Tribe; 2] {
    match race {
        Race::Hyur => [Tribe::Midlander, Tribe::Highlander],
        Race::Elezen => [Tribe::Wildwood, Tribe::Duskwight],
        Race::Lalafell => [Tribe::Plainsfolk, Tribe::Dunesfolk],
        Race::Miqote => [Tribe::Seeker, Tribe::Keeper],
        Race::Roegadyn => [Tribe::SeaWolf, Tribe::Hellsguard],
        Race::AuRa => [Tribe::Raen, Tribe::Xaela],
        Race::Hrothgar => [Tribe::Hellion, Tribe::Lost],
        Race::Viera => [Tribe::Raen, Tribe::Veena],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_race_num() {
        assert_eq!(
            get_race_id(Race::Roegadyn, Tribe::SeaWolf, Gender::Male),
            Some(901)
        );
    }
}
