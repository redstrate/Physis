// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(clippy::needless_range_loop)]

use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::bcn::decode_bc1;
use crate::bcn::decode_bc3;
use crate::bcn::decode_bc5;
use crate::bcn::decode_bc7;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;
use bitflags::bitflags;

#[binrw]
#[derive(Debug)]
struct TextureAttribute(u32);

// Attributes and Format are adapted from Lumina (https://github.com/NotAdam/Lumina/blob/master/src/Lumina/Data/Files/TexFile.cs)
bitflags! {
    impl TextureAttribute : u32 {
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
        const TEXTURE_TYPE2_D_ARRAY = 0x10000000;
        const TEXTURE_TYPE_CUBE = 0x2000000;
        const TEXTURE_TYPE_MASK = 0x3C00000;
        const TEXTURE_SWIZZLE = 0x4000000;
        const TEXTURE_NO_TILED = 0x8000000;
        const TEXTURE_NO_SWIZZLE = 0x80000000;
    }
}

// From https://github.com/aers/FFXIVClientStructs/blob/344f5d488197e9c9d5fd78e92439e7104f25e2e0/FFXIVClientStructs/FFXIV/Client/Graphics/Kernel/Texture.cs#L97
#[binrw]
#[brw(repr = u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)] // NOTE: It's currently allowed to make updating this list not a giant pain
enum TextureFormat {
    L8_UNORM = 0x1130,
    A8_UNORM = 0x1131,
    R8_UNORM = 0x1132,
    R8_UINT = 0x1133,
    R16_UINT = 0x1140,
    R32_UINT = 0x1150,
    R8G8_UNORM = 0x1240,
    B4G4R4A4_UNORM = 0x1440,
    B5G5R5A1_UNORM = 0x1441,
    B8G8R8A8_UNORM = 0x1450,
    B8G8R8X8_UNORM = 0x1451,
    R16_FLOAT = 0x2140,
    R32_FLOAT = 0x2150,
    R16G16_FLOAT = 0x2250,
    R32G32_FLOAT = 0x2260,
    R11G11B10_FLOAT = 0x2350,
    R16G16B16A16_FLOAT = 0x2460,
    R32G32B32A32_FLOAT = 0x2470,
    BC1_UNORM = 0x3420,
    BC2_UNORM = 0x3430,
    BC3_UNORM = 0x3431,
    D16_UNORM = 0x4140,
    D24_UNORM_S8_UINT = 0x4250,
    D16_UNORM_2 = 0x5140,
    D24_UNORM_S8_UINT_2 = 0x5150,
    BC4_UNORM = 0x6120,
    BC5_UNORM = 0x6230,
    BC6H_SF16 = 0x6330,
    BC7_UNORM = 0x6432,
    R16_UNORM = 0x7140,
    R16G16_UNORM = 0x7250,
    R10G10B10A2_UNORM_2 = 0x7350,
    R10G10B10A2_UNORM = 0x7450,
    D24_UNORM_S8_UINT_3 = 0x8250,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
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

#[repr(C)]
#[derive(Clone, Copy)]
pub enum TextureType {
    TwoDimensional,
    ThreeDimensional,
}

/// Texture file, usually with the `.tex` file extension.
///
/// Contains a texture, which optionally be compressed or represent a more complex type like a 3D image.
pub struct Texture {
    /// Type of texture
    pub texture_type: TextureType,
    /// Width of the texture in pixels
    pub width: u32,
    /// Height of the texture in pixels
    pub height: u32,
    /// Depth of the texture in pixels
    pub depth: u32,
    /// Raw RGBA data
    pub rgba: Vec<u8>,
}

type DecodeFunction = fn(&[u8], usize, usize, &mut [u32]) -> Result<(), &'static str>;

impl ReadableFile for Texture {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let header = TexHeader::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        cursor
            .seek(SeekFrom::Start(std::mem::size_of::<TexHeader>() as u64))
            .ok()?;

        let mut src = vec![0u8; buffer.len() - std::mem::size_of::<TexHeader>()];
        cursor.read_exact(src.as_mut_slice()).ok()?;

        let mut dst: Vec<u8>;

        match header.format {
            TextureFormat::B4G4R4A4_UNORM => {
                dst =
                    vec![
                        0u8;
                        header.width as usize * header.height as usize * header.depth as usize * 4
                    ];

                let mut offset = 0;
                let mut dst_offset = 0;

                for _ in 0..header.width as usize * header.height as usize {
                    let short: u16 = ((src[offset] as u16) << 8) | src[offset + 1] as u16;

                    let src_b = short & 0xF;
                    let src_g = (short >> 4) & 0xF;
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
            TextureFormat::B8G8R8A8_UNORM => {
                dst =
                    vec![
                        0u8;
                        header.width as usize * header.height as usize * header.depth as usize * 4
                    ];

                let mut offset = 0;

                for _ in 0..header.width as usize * header.height as usize * header.depth as usize {
                    let src_b = src[offset];
                    let src_g = src[offset + 1];
                    let src_r = src[offset + 2];
                    let src_a = src[offset + 3];

                    dst[offset] = src_r;
                    dst[offset + 1] = src_g;
                    dst[offset + 2] = src_b;
                    dst[offset + 3] = src_a;

                    offset += 4;
                }
            }
            TextureFormat::BC1_UNORM => {
                dst = Texture::decode(
                    &src,
                    header.width as usize,
                    header.height as usize * header.depth as usize,
                    decode_bc1,
                );
            }
            TextureFormat::BC3_UNORM => {
                dst = Texture::decode(
                    &src,
                    header.width as usize,
                    header.height as usize * header.depth as usize,
                    decode_bc3,
                );
            }
            TextureFormat::BC5_UNORM => {
                dst = Texture::decode(
                    &src,
                    header.width as usize,
                    header.height as usize * header.depth as usize,
                    decode_bc5,
                );
            }
            TextureFormat::BC7_UNORM => {
                dst = Texture::decode(
                    &src,
                    header.width as usize,
                    header.height as usize * header.depth as usize,
                    decode_bc7,
                );
            }
            _ => {
                println!("Unsupported texture format {:?}!", header.format);
                return None;
            }
        }

        Some(Texture {
            texture_type: if header.attribute.contains(TextureAttribute::TEXTURE_TYPE3_D) {
                TextureType::ThreeDimensional
            } else {
                TextureType::TwoDimensional
            },
            width: header.width as u32,
            height: header.height as u32,
            depth: header.depth as u32,
            rgba: dst,
        })
    }
}

impl Texture {
    /// Converts an existing texture from `src_platform` to `dst_platform`.
    pub fn convert_existing(
        src_platform: Platform,
        buffer: ByteSpan,
        dst_platform: Platform,
    ) -> ByteBuffer {
        // Read the header from src_platform.
        let mut src_cursor = Cursor::new(buffer);
        let src_header =
            TexHeader::read_options(&mut src_cursor, src_platform.endianness(), ()).unwrap();

        // Write the new header on top of the old one.
        let mut dst_cursor = Cursor::new(buffer.to_vec());
        src_header
            .write_options(&mut dst_cursor, dst_platform.endianness(), ())
            .unwrap();

        dst_cursor.into_inner()
    }

    fn decode(src: &[u8], width: usize, height: usize, decode_func: DecodeFunction) -> Vec<u8> {
        let mut image: Vec<u32> = vec![0; width * height];
        decode_func(src, width, height, &mut image).unwrap();

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
        Texture::from_existing(Platform::Win32, &read(d).unwrap());
    }
}
