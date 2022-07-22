use binrw::binread;

#[binread]
#[br(repr(u8))]
#[repr(u8)]
pub enum Language {
    None,
    Japanese,
    English,
    German,
    French,
    ChineseSimplified,
    ChineseTraditional,
    Korean,
}

pub fn get_language_code(lang: &Language) -> &'static str {
    match &lang {
        Language::None => "",
        Language::Japanese => "ja",
        Language::English => "en",
        Language::German => "de",
        Language::French => "fr",
        Language::ChineseSimplified => "chs",
        Language::ChineseTraditional => "cht",
        Language::Korean => "ko"
    }
}