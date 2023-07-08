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

mod internal_race {
    use paste::paste;

    use crate::define_race_enum;
    use crate::race::Gender;
    use crate::race::Gender::*;
    use crate::race::Race;
    use crate::race::Race::*;
    use crate::race::Subrace;
    use crate::race::Subrace::*;

    define_race_enum! {
        pub enum RaceTest {
            [101](Hyur, Male, Midlander),
            [201](Hyur, Female, Midlander),
            [301](Hyur, Male, Highlander),
            [401](Hyur, Female, Highlander),

            [501](Elezen, Male),
            [601](Elezen, Female),

            [701](Miqote, Male),
            [801](Miqote, Female),

            [901](Roegadyn, Male),
            [1001](Roegadyn, Female),

            [1101](Lalafell, Male),
            [1201](Lalafell, Female),

            [1301](AuRa, Male),
            [1401](AuRa, Female),

            [1501](Hrothgar, Male),
            [1601](Hrothgar, Female),

            [1701](Viera, Male),
            [1801](Viera, Female)
        }
    }
}

/// Gets a proper race identifier (such as 101, for Hyur-Midlander-Males) given a race, a subrace,
/// and a gender.
pub fn get_race_id(race: Race, subrace: Subrace, gender: Gender) -> Option<i32> {
    Some(internal_race::convert_to_internal(race, subrace, gender).unwrap() as i32)
}

pub fn build_skeleton_path(race: Race, subrace: Subrace, gender: Gender) -> String {
    format!(
        "chara/human/c{0:04}/skeleton/base/b0001/skl_c{0:04}b0001.sklb",
        get_race_id(race, subrace, gender).unwrap()
    )
}

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
    use crate::race::internal_race::{convert_to_internal, RaceTest};

    use super::*;

    #[test]
    fn test_convert_to_internal() {
        assert_eq!(
            convert_to_internal(Race::Hyur, Subrace::Midlander, Gender::Female).unwrap(),
            RaceTest::HyurMidlanderFemale
        );
    }
}
