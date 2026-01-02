// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::binrw;

use crate::Error;

/// The playable genders in the game.
#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, Clone, Debug)]
#[repr(u8)]
pub enum Gender {
    Male = 0,
    Female = 1,
}

impl std::convert::TryFrom<u8> for Gender {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            0 => Ok(Self::Male),
            1 => Ok(Self::Female),
            _ => Err(Error::Unknown),
        }
    }
}

/// The playable tribes in the game.
/// Each race has two similar-looking tribes, with the exception of Highlander Hyur which are visually distinct.
#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
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

impl std::convert::TryFrom<u8> for Tribe {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            1 => Ok(Self::Midlander),
            2 => Ok(Self::Highlander),
            3 => Ok(Self::Wildwood),
            4 => Ok(Self::Duskwight),
            5 => Ok(Self::Plainsfolk),
            6 => Ok(Self::Dunesfolk),
            7 => Ok(Self::Seeker),
            8 => Ok(Self::Keeper),
            9 => Ok(Self::SeaWolf),
            10 => Ok(Self::Hellsguard),
            11 => Ok(Self::Raen),
            12 => Ok(Self::Xaela),
            13 => Ok(Self::Hellion),
            14 => Ok(Self::Lost),
            15 => Ok(Self::Rava),
            16 => Ok(Self::Veena),
            _ => Err(Error::Unknown),
        }
    }
}

/// The playable races in the game.
#[binrw]
#[brw(repr = u8)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u8)]
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

impl std::convert::TryFrom<u8> for Race {
    type Error = crate::Error;

    fn try_from(value: u8) -> Result<Self, Error> {
        match value {
            1 => Ok(Self::Hyur),
            2 => Ok(Self::Elezen),
            3 => Ok(Self::Lalafell),
            4 => Ok(Self::Miqote),
            5 => Ok(Self::Roegadyn),
            6 => Ok(Self::AuRa),
            7 => Ok(Self::Hrothgar),
            8 => Ok(Self::Viera),
            _ => Err(Error::Unknown),
        }
    }
}

/// Gets a proper race identifier (such as 101, for Hyur-Midlander-Males) given a race, a tribe,
/// and a gender.
pub fn get_race_id(race: Race, tribe: Tribe, gender: Gender) -> Option<i32> {
    if !get_supported_tribes(race).contains(&tribe) {
        return None;
    }

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
        Race::Hrothgar => match gender {
            Gender::Male => Some(1501),
            Gender::Female => Some(1601),
        },
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
        // valid
        assert_eq!(
            get_race_id(Race::Roegadyn, Tribe::SeaWolf, Gender::Male),
            Some(901)
        );
        // invalid
        assert_eq!(
            get_race_id(Race::Roegadyn, Tribe::Midlander, Gender::Male),
            None
        );
    }
}
