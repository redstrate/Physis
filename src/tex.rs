use std::cmp::min;
use std::io::{Cursor, Read, Seek, SeekFrom};
use binrw::binread;
use crate::gamedata::MemoryBuffer;
use binrw::BinRead;
use bitflags::bitflags;
use texpresso::Format;

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

#[binread]
#[br(repr = u32)]
#[derive(Debug)]
enum TextureFormat {
    B8G8R8A8 = 0x1450,
    BC1 = 0x3420,
    BC5 = 0x3431
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
    width: u32,
    height: u32,
    rgba: Vec<u8>
}

impl Texture {
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<Texture> {
        let mut cursor = Cursor::new(buffer);
        let header = TexHeader::read(&mut cursor).unwrap();

        // TODO: Adapted from Lumina, but this really can be written better...
        let mut texture_data_size = vec![];
        texture_data_size.resize(min(13, header.mip_levels as usize), 0);
        let size = texture_data_size.len();
        for i in 0..size - 1 {
            texture_data_size[i] = header.offset_to_surface[i + 1] - header.offset_to_surface[i];
        }
        texture_data_size[size - 1] = (buffer.len() - header.offset_to_surface[size - 1] as usize) as u32;

        cursor.seek(SeekFrom::Start(header.offset_to_surface[0] as u64)).ok()?;

        let mut src = vec![0u8; texture_data_size.iter().sum::<u32>() as usize];
        cursor.read_exact(src.as_mut_slice()).ok()?;

        let mut dst : Vec<u8> = vec![0u8; (header.width as usize * header.height as usize * 4) as usize];

        match header.format {
            TextureFormat::B8G8R8A8 => {
                dst.copy_from_slice(&src);
            }
            TextureFormat::BC1 => {
                let format = Format::Bc1;
                format.decompress(&src, header.width as usize, header.height as usize, dst.as_mut_slice());
            }
            TextureFormat::BC5 => {
                let format = Format::Bc3;
                format.decompress(&src, header.width as usize, header.height as usize, dst.as_mut_slice());
            }
        }

        Some(Texture {
            width: header.width as u32,
            height: header.height as u32,
            rgba: dst
        })
    }
}