use binrw::binread;

use super::GameInstanceObject;

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct AetheryteInstanceObject {
    pub parent_data: GameInstanceObject,
    pub bound_instance_id: u32,
    padding: u32,
}
