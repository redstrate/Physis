// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs;
use std::path::Path;

#[cfg(test)]
use binrw::BinWrite;
use binrw::{Endian, binrw};
use strum::EnumIter;
use strum_macros::{Display, FromRepr};

#[binrw]
#[brw(repr(u8))]
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Language the game data is written for.
///
/// Keep in mind that the selection of languages vary depending on the client's region.
pub enum Language {
    /// Used for data that is language-agnostic.
    None,
    /// Japanese language. Only available in the Global client.
    Japanese,
    /// English language. Only available in the Global client.
    English,
    /// German language. Only available in the Global client.
    German,
    /// French language. Only available in the Global client.
    French,
    /// Chinese (Simplified) language. Only available in the Chinese client.
    ChineseSimplified,
    /// Chinese (Traditional) language. Only available in the Chinese client.
    ChineseTraditional,
    /// Korean language. Only available in the Korean client.
    Korean,
    /// (Traditional) Chinese language. Only available in the Taiwanese client.
    TraditionalChinese,
}

impl Language {
    /// Returns the shorthand language code for `language`.
    ///
    /// For example, English becomes "en".
    pub fn shortname(&self) -> &'static str {
        // NOTE: Keep in sync with `from_shortname`.
        match self {
            Language::None => "",
            Language::Japanese => "ja",
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::ChineseSimplified => "chs",
            Language::ChineseTraditional => "cht",
            Language::Korean => "ko",
            Language::TraditionalChinese => "tc",
        }
    }

    /// Returns the language for a given shorthand, if valid.
    pub fn from_shortname(shortname: &str) -> Self {
        // NOTE: Keep in sync with `shortname`.
        match shortname {
            "ja" => Language::Japanese,
            "en" => Language::English,
            "de" => Language::German,
            "fr" => Language::French,
            "chs" => Language::ChineseSimplified,
            "cht" => Language::ChineseTraditional,
            "ko" => Language::Korean,
            "tc" => Language::TraditionalChinese,
            _ => Language::None,
        }
    }
}

/// Read a version file.
///
/// It intentionally reads whitespace as the game reads those characters too.
// TODO: use version type
pub fn read_version(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}

/// Platform used for game data.
#[binrw]
#[brw(repr = u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
#[repr(C)]
pub enum Platform {
    /// Windows and macOS.
    Win32 = 0x0,
    /// Playstation 3.
    PS3 = 0x1,
    /// Playstation 4.
    PS4 = 0x2,
    /// Playstation 5.
    PS5 = 0x3,
    /// Xbox One.
    Xbox = 0x4,
    /// Nintendo Switch 2.
    NS2 = 0x5,
}

impl Platform {
    /// Returns the short-hand codename for this platform.
    ///
    /// For example, `Platform::Win32` becomes "win32".
    pub fn shortname(&self) -> &'static str {
        match self {
            Platform::Win32 => "win32",
            Platform::PS3 => "ps3",
            Platform::PS4 => "ps4",
            Platform::PS5 => "ps5",
            Platform::Xbox => "lys",
            Platform::NS2 => "obe",
        }
    }

    /// Returns the endianness for this platform.
    pub(crate) fn endianness(&self) -> Endian {
        match self {
            Platform::PS3 => Endian::Big,
            _ => Endian::Little,
        }
    }
}

use std::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
};

/// Represents a game version, e.g. "2025.02.27.0000.0000".
///
/// Unlike a normal string, this can sort itself in a sensible way.
#[derive(PartialEq, Eq, PartialOrd)]
pub struct Version<'a>(pub &'a str);

#[derive(PartialEq, Eq, Ord, PartialOrd)]
struct VersionParts {
    year: i32,
    month: i32,
    day: i32,
    patch1: i32,
    patch2: i32,
}

impl VersionParts {
    fn new(version: &str) -> Self {
        let parts: Vec<&str> = version.split('.').collect();

        Self {
            year: parts[0].parse::<i32>().unwrap(),
            month: parts[1].parse::<i32>().unwrap(),
            day: parts[2].parse::<i32>().unwrap(),
            patch1: parts[3].parse::<i32>().unwrap(),
            patch2: parts[4].parse::<i32>().unwrap(),
        }
    }
}

impl Display for Version<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Ord for Version<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        let our_version_parts = VersionParts::new(self.0);
        let their_version_parts = VersionParts::new(other.0);

        our_version_parts.cmp(&their_version_parts)
    }
}

/// A file that can be parsed from its serialized byte form.
///
/// This should be implemented for all types readable from SqPack.
pub trait ReadableFile: Sized {
    /// Read an existing file.
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self>;
}

/// A file that can be written back to its serialized byte form.
///
/// This should be implemented for all types readable from SqPack, on a best-effort basis.
pub trait WritableFile: Sized {
    /// Writes data back to a buffer.
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer>;
}

/// Used for basic sanity checking tests in other modules.
#[cfg(test)]
pub fn pass_random_invalid<T: ReadableFile>() {
    let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("resources/tests");
    d.push("random");

    // Feeding it invalid data should not panic
    // Note that we don't check the Option currently, because some types like Hwc return Some regardless.
    let _ = T::from_existing(
        Platform::Win32,
        &std::fs::read(d).expect("Could not read random test file"),
    );
}

/// Helper to ensure that type `T` is written to `EXPECTED_SIZE`.
#[cfg(test)]
pub fn ensure_size<T: BinWrite + Default, const EXPECTED_SIZE: usize>()
where
    for<'a> T: BinWrite<Args<'a> = ()> + 'a + Default,
{
    use std::io::Cursor;

    let mut cursor = Cursor::new(Vec::new());
    let instance = T::default();
    instance.write_ne(&mut cursor).expect("Failed to write!");

    assert_eq!(cursor.position() as usize, EXPECTED_SIZE);
}

/// A continuous block of memory which is not owned, and comes either from an in-memory location or from a file.
pub type ByteSpan<'a> = &'a [u8];

/// A continuous block of memory which is owned.
pub type ByteBuffer = Vec<u8>;

/// Names for the rows of the Excel sheet of the same name.
///
/// Based off of <https://github.com/aers/FFXIVClientStructs/blob/main/FFXIVClientStructs/FFXIV/Client/Enums/TerritoryIntendedUse.cs>.
#[repr(u8)]
#[derive(FromRepr, Display, Clone, Copy, PartialEq)]
pub enum TerritoryIntendedUse {
    /// Towns such as Limsa Lominsa.
    Town = 0,
    /// Open world zones such as everything out of towns.
    Overworld = 1,
    /// Inn rooms.
    Inn = 2,
    /// Dungeon zones and other misc duties like Air Force One.
    Dungeon = 3,
    /// Variant dungeons like The Sil'dihn Subterrane.
    VariantDungeon = 4,
    /// Jail zones like Mordion Gaol.
    MordionGaol = 5,
    /// Copies of Towns that are only during the opening.
    OpeningArea = 6,
    /// Rarely seen "lobby zones", such as the Phantom Village for Occult Crescent.
    BeforeTrialDung = 7,
    /// Zones used in Alliance Raids.
    AllianceRaid = 8,
    /// Used for (pre-Endwalker?) quest battles.
    PreEwOverworldQuestBattle = 9,
    /// Trial battles.
    Trial = 10,
    /// Currently unused.
    Unknown11 = 11,
    /// ???
    WaitingRoom = 12,
    /// ???
    HousingOutdoor = 13,
    /// ???
    HousingIndoor = 14,
    /// ???
    SoloOverworldInstances = 15,
    /// Fighting arenas for raids like.
    Raid1 = 16,
    /// Seen in at least AAC Heavyweight M1 (Savage)
    Raid2 = 17,
    /// Zones used for Frontline PvP.
    Frontline = 18,
    /// Now unused.
    ChocoboSquareOld = 19,
    /// ???
    ChocoboRacing = 20,
    /// Used for the only Ishgard Restoration zone, the Firamament.
    Firmament = 21,
    /// The Sanctum of the Twelve zone used for weddings.
    SanctumOfTheTwelve = 22,
    /// Gold Saucer zones.
    GoldSaucer = 23,
    /// Now unused.
    OriginalStepsOfFaith = 24,
    /// ???
    LordOfVerminion = 25,
    /// ???
    ExploratoryMissions = 26,
    /// Used for the Hall of Novice tutorials.
    HallOfTheNovice = 27,
    /// Zones used for Crystalline Conflict PvP.
    CrystallineConflict = 28,
    /// Used for events like Solo Duties.
    SoloDuty = 29,
    /// The barracks zones of grand companies.
    GrandCompanyBarracks = 30,
    /// Zones used for Deep Dugeons, e.g. Palace of the Dead.
    DeepDungeon = 31,
    /// Used for zones only accessible seasonally, like Starlight Halls.
    Seasonal = 32,
    /// Treasure dungeons like Vault Oneiron.
    TreasureMapInstance = 33,
    /// ???
    SeasonalInstancedArea = 34,
    /// ???
    TripleTriadBattleHall = 35,
    /// Used for raids like The Cloud of Darkness (Chaotic).
    ChaoticRaid = 36,
    /// ???
    CrystallineConflictCustomMatch = 37,
    /// Diadem
    HuntingGrounds = 38,
    /// Used in Rival Wings content.
    RivalWings = 39,
    /// Mordion Gaol (The Rising 2017), Frondale's Home for Friendless Foundlings, Starlight Stalls.
    Seasonal2 = 40,
    /// Eureka zones.
    Eureka = 41,
    /// ???
    Unknown42 = 42,
    /// ???
    TheCalamityRetold = 43,
    /// Leap of Faith zones.
    LeapOfFaith = 44,
    /// ???
    MaskedCarnival = 45,
    /// Zones used for Ocean Fishing.
    OceanFishing = 46,
    /// ???
    Diadem = 47,
    /// ???
    Bozja = 48,
    /// Island Sanctuary zones.
    IslandSanctuary = 49,
    /// ???
    TripleTriadOpenTournament = 50,
    /// Used in the Triple Triad Invitational Parlor duty.
    TripleTriadInvitationalParlor = 51,
    /// ???
    DelubrumReginae = 52,
    /// ???
    DelubrumReginaeSavage = 53,
    /// Propylaion and Ultima Thule
    EndwalkerMsqSoloOverworld = 54,
    /// Currently unused.
    Unknown55 = 55,
    /// ???
    Elysion = 56,
    /// Criterion Dungeon zones.
    CriterionDungeon = 57,
    /// Savage Criterion Dungeon zones.
    CriterionDungeonSavage = 58,
    /// Bean containment zones.
    Blunderville = 59,
    /// Cosmic Exploration zones.
    CosmicExploration = 60,
    /// Occult Crescent zones.
    OccultCrescent = 61,
    /// ???
    Unknown62 = 62,
    /// Lilyswim (Hatching-tide 2026)
    Seasonal3 = 63,
    /// ???
    AirForceOne = 64,
    /// ???
    KeyboundBrawler = 65,
}

/// 32-bit color.
#[binrw]
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Color {
    /// Red component.
    pub red: u8,
    /// Green component.
    pub green: u8,
    /// Blue component.
    pub blue: u8,
    /// Alpha component.
    pub alpha: u8,
}

/// 32-bit color with an extra 32-bit intensity component.
#[binrw]
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct ColorIntensity {
    /// Red component.
    pub red: u8,
    /// Green component.
    pub green: u8,
    /// Blue component.
    pub blue: u8,
    /// Alpha component.
    pub alpha: u8,
    /// Intensity of the color?
    pub intensity: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq() {
        assert!(Version("2025.02.27.0000.0000") == Version("2025.02.27.0000.0000"));
        assert!(Version("2025.01.20.0000.0000") != Version("2025.02.27.0000.0000"));
    }

    #[test]
    fn test_ordering() {
        // year
        assert!(Version("2025.02.27.0000.0000") > Version("2024.02.27.0000.0000"));

        // month
        assert!(Version("2025.03.27.0000.0000") > Version("2025.02.27.0000.0000"));

        // day
        assert!(Version("2025.02.28.0000.0000") > Version("2025.02.27.0000.0000"));

        // patch1
        assert!(Version("2025.02.27.1000.0000") > Version("2025.02.27.0000.0000"));

        // patch2
        assert!(Version("2025.02.27.0000.1000") > Version("2025.02.27.0000.0000"));
    }

    #[test]
    fn test_version() {
        let mut dir = std::env::temp_dir();
        dir.push("test.ver");
        if dir.exists() {
            std::fs::remove_file(&dir).unwrap();
        }

        assert_eq!(read_version(&dir), None);

        std::fs::write(&dir, "2023.09.15.0000.0000").unwrap();
        assert_eq!(read_version(&dir), Some("2023.09.15.0000.0000".to_string()));

        std::fs::write(&dir, "2023.09.15.0000.0000\r\n").unwrap();
        assert_eq!(
            read_version(&dir),
            Some("2023.09.15.0000.0000\r\n".to_string())
        );
    }
}
