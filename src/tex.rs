// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::needless_range_loop)]

use std::cmp::min;
use std::io::{Cursor, Read, Seek, SeekFrom};

use binrw::BinRead;
use binrw::binrw;
use bitflags::bitflags;
use texpresso::Format;
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
    B8G8R8A8 = 0x1450,
    BC1 = 0x3420,
    BC5 = 0x3431,
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

impl Texture {
    /// Reads an existing TEX file
    pub fn from_existing(buffer: ByteSpan) -> Option<Texture> {
        let mut cursor = Cursor::new(buffer);
        let header = TexHeader::read(&mut cursor).unwrap();

        // TODO: Adapted from Lumina, but this really can be written better...
        let mut texture_data_size = vec![0; min(13, header.mip_levels as usize)];
        let size = texture_data_size.len();
        for i in 0..size - 1 {
            texture_data_size[i] = header.offset_to_surface[i + 1] - header.offset_to_surface[i];
        }

        texture_data_size[size - 1] =
            (buffer.len() - header.offset_to_surface[size - 1] as usize) as u32;

        cursor
            .seek(SeekFrom::Start(header.offset_to_surface[0] as u64))
            .ok()?;

        let mut src = vec![0u8; texture_data_size.iter().sum::<u32>() as usize];
        cursor.read_exact(src.as_mut_slice()).ok()?;

        let mut dst;

        match header.format {
            TextureFormat::B8G8R8A8 => {
                dst =
                    vec![0u8; texture_data_size.iter().sum::<u32>() as usize];

                dst.copy_from_slice(&src);
            }
            TextureFormat::BC1 => {
                dst = vec![0u8; header.width as usize * header.height as usize * 4];

                let format = Format::Bc1;
                format.decompress(
                    &src,
                    header.width as usize,
                    header.height as usize,
                    dst.as_mut_slice(),
                );
            }
            TextureFormat::BC5 => {
                dst = vec![0u8; header.width as usize * header.height as usize * 4];

                let format = Format::Bc3;
                format.decompress(
                    &src,
                    header.width as usize,
                    header.height as usize,
                    dst.as_mut_slice(),
                );
            }
        }

        Some(Texture {
            width: header.width as u32,
            height: header.height as u32,
            rgba: dst,
        })
    }
}
