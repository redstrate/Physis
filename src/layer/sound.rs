use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct SoundInstanceObject {
    pub sound_effect_param: i32,
    pub asset_path_offset: u32,
    // TODO: read separam
}
