use std::io::Cursor;
use binrw::binread;
use crate::gamedata::MemoryBuffer;
use binrw::BinRead;

#[binread]
#[derive(Debug)]
struct TexHeader {
    attribute : u32,
    format: u32,

    width : u16,
    height : u16,
    depth : u16,
    mip_levels : u16,

    lod_offsets : [u32; 3],
    offset_to_surface : [u32; 13]
}

pub struct Texture {

}

impl Texture {
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<Texture> {
        let mut cursor = Cursor::new(buffer);
        let header = TexHeader::read(&mut cursor).unwrap();

        println!("{:#?}", header);

        None
    }
}