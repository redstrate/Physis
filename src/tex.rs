use std::io::Cursor;
use binrw::binread;
use crate::gamedata::MemoryBuffer;
use binrw::BinRead;
use bitflags::bitflags;

// Attributes and Format are adapted from Lumina (https://github.com/NotAdam/Lumina/blob/master/src/Lumina/Data/Files/TexFile.cs)
bitflags! {
    #[binread]
    struct TextureAttribute : u32 {
        const DiscardPerFrame = 0x1;
        const DiscardPerMap = 0x2;

        const Managed = 0x4;
        const UserManaged = 0x8;
        const CpuRead = 0x10;
        const LocationMain = 0x20;
        const NoGpuRead = 0x40;
        const AlignedSize = 0x80;
        const EdgeCulling = 0x100;
        const LocationOnion = 0x200;
        const ReadWrite = 0x400;
        const Immutable = 0x800;

        const TextureRenderTarget = 0x100000;
        const TextureDepthStencil = 0x200000;
        const TextureType1D = 0x400000;
        const TextureType2D = 0x800000;
        const TextureType3D = 0x1000000;
        const TextureTypeCube = 0x2000000;
        const TextureTypeMask = 0x3C00000;
        const TextureSwizzle = 0x4000000;
        const TextureNoTiled = 0x8000000;
        const TextureNoSwizzle = 0x80000000;
    }
}

bitflags! {
    #[binread]
    struct TextureFormat : u32 {
        const TypeShift = 0xC;
        const TypeMask = 0xF000;
        const ComponentShift = 0x8;
        const ComponentMask = 0xF00;
        const BppShift = 0x4;
        const BppMask = 0xF0;
        const EnumShift = 0x0;
        const EnumMask = 0xF;
        const TypeInteger = 0x1;
        const TypeFloat = 0x2;
        const TypeDxt = 0x3;
        const TypeBc123 = 0x3;
        const TypeDepthStencil = 0x4;
        const TypeSpecial = 0x5;
        const TypeBc57 = 0x6;

        const L8 = 0x1130;
        const A8 = 0x1131;
        const B4G4R4A4 = 0x1440;
        const B5G5R5A1 = 0x1441;
        const B8G8R8A8 = 0x1450;
        const B8G8R8X8 = 0x1451;

        const R32F = 0x2150;
        const R16G16F = 0x2250;
        const R32G32F = 0x2260;
        const R16G16B16A16F = 0x2460;
        const R32G32B32A32F = 0x2470;

        const BC1 = 0x3420;
        const BC2 = 0x3430;
        const BC3 = 0x3431;
        const BC5 = 0x6230;
        const BC7 = 0x6432;

        const D16 = 0x4140;
        const D24S8 = 0x4250;

        const Null = 0x5100;
        const Shadow16 = 0x5140;
        const Shadow24 = 0x5150;
    }
}

#[binread]
#[derive(Debug)]
struct TexHeader {
    attribute : TextureAttribute,
    format: TextureFormat,

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