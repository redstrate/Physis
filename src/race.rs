// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
/// Gender of the character.
pub enum Gender {
    Male,
    Female,
}

#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
/// The race's "subrace". Each race has two subraces, which are actually identical (even down to the ids!)
/// with the exception of Hyurs, which have two unique subraces that are really two separate races.
pub enum Subrace {
    Midlander,
    Highlander,
    Wildwood,
    Duskwight,
    Plainsfolk,
    Dunesfolk,
    Seeker,
    Keeper,
    SeaWolf,
    Hellsguard,
    Raen,
    Xaela,
    Hellion,
    Lost,
    Rava,
    Veena,
}

#[derive(PartialEq, Eq, Debug)]
#[repr(u8)]
/// The major races of Eorzea.
pub enum Race {
    Hyur,
    Elezen,
    Lalafell,
    Miqote,
    Roegadyn,
    AuRa,
    Hrothgar,
    Viera,
}

/// Gets a proper race identifier (such as 101, for Hyur-Midlander-Males) given a race, a subrace,
/// and a gender.
pub fn get_race_id(race: Race, subrace: Subrace, gender: Gender) -> Option<i32> {
    // TODO: should we check for invalid subraces like the Hyur branch does?
    match race {
        Race::Hyur => {
            match subrace {
                Subrace::Midlander => {
                    match gender {
                        Gender::Male => Some(101),
                        Gender::Female => Some(201)
                    }
                }
                Subrace::Highlander => {
                    match gender {
                        Gender::Male => Some(301),
                        Gender::Female => Some(401)
                    }
                }
                _ => None
            }
        }
        Race::Elezen => {
            match gender {
                Gender::Male => Some(501),
                Gender::Female => Some(601)
            }
        }
        Race::Lalafell => {
            match gender {
                Gender::Male => Some(501),
                Gender::Female => Some(601)
            }
        }
        Race::Miqote => {
            match gender {
                Gender::Male => Some(701),
                Gender::Female => Some(801)
            }
        }
        Race::Roegadyn => {
            match gender {
                Gender::Male => Some(901),
                Gender::Female => Some(1001)
            }
        }
        Race::AuRa => {
            match gender {
                Gender::Male => Some(1301),
                Gender::Female => Some(1401)
            }
        }
        Race::Hrothgar => {
            match gender {
                Gender::Male => Some(1501),
                Gender::Female => Some(1601) // TODO: is this accurate as of dawntrail?
            }
        }
        Race::Viera => {
            match gender {
                Gender::Male => Some(1701),
                Gender::Female => Some(1801)
            }
        }
    }
}

/// Builds the path to the skeleton (sklb) file for a given `race`, `subrace` and `gender`.
pub fn build_skeleton_path(race: Race, subrace: Subrace, gender: Gender) -> String {
    format!(
        "chara/human/c{0:04}/skeleton/base/b0001/skl_c{0:04}b0001.sklb",
        get_race_id(race, subrace, gender).unwrap()
    )
}

/// Returns the two subraces associated with a given `race`. For example, `Hyur` would return `[Midlander, Highlander]`.
pub fn get_supported_subraces(race: Race) -> [Subrace; 2] {
    match race {
        Race::Hyur => [Subrace::Midlander, Subrace::Highlander],
        Race::Elezen => [Subrace::Wildwood, Subrace::Duskwight],
        Race::Lalafell => [Subrace::Plainsfolk, Subrace::Dunesfolk],
        Race::Miqote => [Subrace::Seeker, Subrace::Keeper],
        Race::Roegadyn => [Subrace::SeaWolf, Subrace::Hellsguard],
        Race::AuRa => [Subrace::Raen, Subrace::Xaela],
        Race::Hrothgar => [Subrace::Hellion, Subrace::Lost],
        Race::Viera => [Subrace::Raen, Subrace::Veena]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_race_num() {
        assert_eq!(
            get_race_id(Race::Roegadyn, Subrace::SeaWolf, Gender::Male),
            Some(901)
        );
    }
}
