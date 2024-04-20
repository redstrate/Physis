// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::needless_range_loop)]

use std::io::{Cursor, Read, Seek, SeekFrom};

use binrw::BinRead;
use binrw::binrw;
use bitflags::bitflags;
use texture2ddecoder::{decode_bc1, decode_bc3, decode_bc5};
use crate::ByteSpan;

// Attributes and Format are adapted from Lumina (https://github.com/NotAdam/Lumina/blob/master/src/Lumina/Data/Files/TexFile.cs)
bitflags! {
    #[binrw]
    struct TextureAttribute : u32 {
        const DISCARD_PER_FRAME = 0x1;
        const DISCARD_PER_MAP = 0x2;

        const MANAGED = 0x4;
        const USER_MANAGED = 0x8;
        const CPU_READ = 0x10;
        const LOCATION_MAIN = 0x20;
        const NO_GPU_READ = 0x40;
        const ALIGNED_SIZE = 0x80;
        const EDGE_CULLING = 0x100;
        const LOCATION_ONION = 0x200;
        const READ_WRITE = 0x400;
        const IMMUTABLE = 0x800;

        const TEXTURE_RENDER_TARGET = 0x100000;
        const TEXTURE_DEPTH_STENCIL = 0x200000;
        const TEXTURE_TYPE1_D = 0x400000;
        const TEXTURE_TYPE2_D = 0x800000;
        const TEXTURE_TYPE3_D = 0x1000000;
        const TEXTURE_TYPE_CUBE = 0x2000000;
        const TEXTURE_TYPE_MASK = 0x3C00000;
        const TEXTURE_SWIZZLE = 0x4000000;
        const TEXTURE_NO_TILED = 0x8000000;
        const TEXTURE_NO_SWIZZLE = 0x80000000;
    }
}

#[binrw]
#[brw(repr = u32)]
#[derive(Debug)]
enum TextureFormat {
    B4G4R4A4 = 0x1440,
    B8G8R8A8 = 0x1450,
    BC1 = 0x3420,
    BC3 = 0x3431,
    BC5 = 0x6230,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
#[brw(little)]
struct TexHeader {
    attribute: TextureAttribute,
    format: TextureFormat,

    width: u16,
    height: u16,
    depth: u16,
    mip_levels: u16,

    lod_offsets: [u32; 3],
    offset_to_surface: [u32; 13],
}

pub struct Texture {
    /// Width of the texture in pixels
    pub width: u32,
    /// Height of the texture in pixels
    pub height: u32,
    /// Raw RGBA data
    pub rgba: Vec<u8>,
}

type DecodeFunction = fn(&[u8], usize, usize, &mut [u32]) -> Result<(), &'static str>;

impl Texture {
    /// Reads an existing TEX file
    pub fn from_existing(buffer: ByteSpan) -> Option<Texture> {
        let mut cursor = Cursor::new(buffer);
        let header = TexHeader::read(&mut cursor).unwrap();

        cursor
            .seek(SeekFrom::Start(std::mem::size_of::<TexHeader>() as u64))
            .ok()?;

        let mut src = vec![0u8; buffer.len() - std::mem::size_of::<TexHeader>()];
        cursor.read_exact(src.as_mut_slice()).ok()?;

        let mut dst : Vec<u8>;

        match header.format {
            TextureFormat::B4G4R4A4 => {
                dst = vec![0u8; header.width as usize * header.height as usize * 4];

                let mut offset = 0;
                let mut dst_offset = 0;

                for _ in 0..header.width * header.height {
                    let short: u16 = ((src[offset] as u16) << 8) | src[offset + 1] as u16;

                    let src_b = short & 0xF;
                    let src_g= (short >> 4) & 0xF;
                    let src_r = (short >> 8) & 0xF;
                    let src_a = (short >> 12) & 0xF;

                    dst[dst_offset] = (17 * src_r) as u8;
                    dst[dst_offset + 1] = (17 * src_g) as u8;
                    dst[dst_offset + 2] = (17 * src_b) as u8;
                    dst[dst_offset + 3] = (17 * src_a) as u8;

                    offset += 2;
                    dst_offset += 4;
                }
            }
            TextureFormat::B8G8R8A8 => {
                dst = src; // TODO: not correct, of course
            }
            TextureFormat::BC1 => {
                dst = Texture::decode(&src, header.width as usize, header.height as usize, decode_bc1);
            }
            TextureFormat::BC3 => {
                dst = Texture::decode(&src, header.width as usize, header.height as usize, decode_bc3);
            }
            TextureFormat::BC5 => {
                dst = Texture::decode(&src, header.width as usize, header.height as usize, decode_bc5);
            }
        }

        Some(Texture {
            width: header.width as u32,
            height: header.height as u32,
            rgba: dst,
        })
    }

    fn decode(src: &[u8], width: usize, height: usize, decode_func: DecodeFunction) -> Vec<u8> {
        let mut image: Vec<u32> = vec![0; (width * height) as usize];
        decode_func(
            &src,
            width as usize,
            height as usize,
            &mut image,
        )
            .unwrap();

        image
            .iter()
            .flat_map(|x| {
                let v = x.to_le_bytes();
                [v[2], v[1], v[0], v[3]]
            })
            .collect::<Vec<u8>>()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        // Feeding it invalid data should not panic
        Texture::from_existing(&read(d).unwrap());
    }
}
