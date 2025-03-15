use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct RelativePositions {
    pos: i32,
    pos_count: i32,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
#[allow(dead_code)] // most of the fields are unused at the moment
pub struct Transformation {
    pub translation: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
pub struct ColorHDRI {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
    pub intensity: f32,
}
