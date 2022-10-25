use std::fs;
use std::path::Path;
use binrw::binrw;

#[binrw]
#[brw(repr(u8))]
#[repr(u8)]
/// The language the game data is written for.
pub enum Language {
    /// Used for data that is language-agnostic, such as item data.
    None,
    /// Japanese language.
    Japanese,
    /// English language.
    English,
    /// German language.
    German,
    /// French language.
    French,
    /// Chinese (Simplified) language.
    ChineseSimplified,
    /// Chinese (Traditional) language.
    ChineseTraditional,
    /// Korean language.
    Korean,
}

/// Returns the shorthand language code for `language`. For example, English becomes "en".
pub fn get_language_code(lang: &Language) -> &'static str {
    match &lang {
        Language::None => "",
        Language::Japanese => "ja",
        Language::English => "en",
        Language::German => "de",
        Language::French => "fr",
        Language::ChineseSimplified => "chs",
        Language::ChineseTraditional => "cht",
        Language::Korean => "ko",
    }
}

#[binrw]
#[brw(repr = i16)]
#[derive(Debug, PartialEq, Eq)]
pub enum Region {
    Global = -1, // TODO: find patch codes for other regions :-)
}

/// Reads a version file.
pub fn read_version(p: &Path) -> Option<String> {
    fs::read_to_string(p).ok()
}
