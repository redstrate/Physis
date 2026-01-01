// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Write;
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::ByteBuffer;
use crate::common::Platform;
use binrw::BinWrite;
use binrw::{BinRead, VecArgs};
use binrw::{BinReaderExt, binrw};

use crate::common_file_operations::{read_bool_from, write_bool_as};
use crate::model::ModelFileHeader;
use crate::sqpack::read_data_block;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq, Eq)]
/// The file type of the data entry.
pub enum FileType {
    /// Empty entry, usually invalid.
    Empty = 1,
    /// Encompasses every file that is not a model or a texture.
    Standard,
    /// Model (.mdl) files.
    Model,
    /// Texture (.tex) files.
    Texture,
}

#[binrw]
#[derive(Debug)]
struct StandardFileBlock {
    #[brw(pad_before = 8)]
    num_blocks: u32,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
struct TextureLodBlock {
    compressed_offset: u32,
    compressed_size: u32,
    decompressed_size: u32,

    block_offset: u32,
    block_count: u32,
}

pub trait AnyNumberType<'a>:
    BinRead<Args<'a> = ()> + BinWrite<Args<'a> = ()> + std::ops::AddAssign + Copy + Default + 'static
{
}

impl<'a, T> AnyNumberType<'a> for T where
    T: BinRead<Args<'a> = ()>
        + BinWrite<Args<'a> = ()>
        + std::ops::AddAssign
        + Copy
        + Default
        + 'static
{
}

#[binrw]
#[derive(Debug)]
pub struct ModelMemorySizes<T: for<'a> AnyNumberType<'a>> {
    pub stack_size: T,
    pub runtime_size: T,

    pub vertex_buffer_size: [T; 3],
    pub edge_geometry_vertex_buffer_size: [T; 3],
    pub index_buffer_size: [T; 3],
}

impl<T: for<'a> AnyNumberType<'a>> ModelMemorySizes<T> {
    pub fn total(&self) -> T {
        let mut total: T = T::default();

        total += self.stack_size;
        total += self.runtime_size;

        for i in 0..3 {
            total += self.vertex_buffer_size[i];
            total += self.edge_geometry_vertex_buffer_size[i];
            total += self.index_buffer_size[i];
        }

        total
    }
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ModelFileBlock {
    pub num_blocks: u32,
    pub num_used_blocks: u32,
    pub version: u32,

    pub uncompressed_size: ModelMemorySizes<u32>,
    pub compressed_size: ModelMemorySizes<u32>,
    pub offset: ModelMemorySizes<u32>,
    pub index: ModelMemorySizes<u16>,
    pub num: ModelMemorySizes<u16>,

    pub vertex_declaration_num: u16,
    pub material_num: u16,
    pub num_lods: u8,

    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub index_buffer_streaming_enabled: bool,
    #[brw(pad_after = 1)]
    #[br(map = read_bool_from::<u8>)]
    #[bw(map = write_bool_as::<u8>)]
    pub edge_geometry_enabled: bool,
}

#[binrw]
#[derive(Debug)]
struct TextureBlock {
    #[br(pad_before = 8)]
    num_blocks: u32,

    #[br(count = num_blocks)]
    lods: Vec<TextureLodBlock>,
}

/// A SqPack file info header. It can optionally contain extra information, such as texture or
/// model data depending on the file type.
#[binrw]
#[derive(Debug)]
struct FileInfo {
    size: u32,
    file_type: FileType,
    file_size: u32,

    #[br(if (file_type == FileType::Standard))]
    #[bw(if (*file_type == FileType::Standard))]
    standard_info: Option<StandardFileBlock>,

    #[br(if (file_type == FileType::Model))]
    #[bw(if (*file_type == FileType::Model))]
    model_info: Option<ModelFileBlock>,

    #[br(if (file_type == FileType::Texture))]
    #[bw(if (*file_type == FileType::Texture))]
    texture_info: Option<TextureBlock>,
}

#[binrw]
pub struct Block {
    #[br(pad_after = 4)]
    offset: i32,
}

#[binrw]
#[derive(Debug)]
#[br(import { x : i32, y : i32 })]
#[br(map = | _ : i32 | if x < 32000 { CompressionMode::Compressed{ compressed_length : x, decompressed_length : y} } else { CompressionMode::Uncompressed { file_size : y } } )]
pub enum CompressionMode {
    // we manually map here, because for this case the enum value is also a raw value we want to extract :-)
    Compressed {
        compressed_length: i32,
        decompressed_length: i32,
    },
    Uncompressed {
        file_size: i32,
    },
}

#[binrw]
#[derive(Debug)]
pub struct BlockHeader {
    #[brw(pad_after = 4)]
    pub size: u32,

    #[br(temp)]
    #[bw(calc = match compression { CompressionMode::Compressed{ compressed_length, .. } => { *compressed_length } CompressionMode::Uncompressed{ .. } => { 32000 }})]
    x: i32,

    #[br(temp)]
    #[bw(calc = match compression { CompressionMode::Compressed{ decompressed_length, .. } => { *decompressed_length } CompressionMode::Uncompressed{ file_size } => { *file_size }})]
    y: i32,

    #[br(args { x, y })]
    #[brw(restore_position)]
    pub compression: CompressionMode,
}

pub struct SqPackData {
    platform: Platform,
    file: std::fs::File,
}

impl SqPackData {
    /// Creates a new reference to an existing dat file.
    pub fn from_existing(platform: Platform, path: &str) -> Option<Self> {
        Some(Self {
            platform,
            file: std::fs::File::open(path).ok()?,
        })
    }

    /// Reads from a certain offset inside of the dat file. This offset will be fixed automatically
    /// by the function.
    ///
    /// If the block of data is successfully parsed, it returns the file data - otherwise is None.
    pub fn read_from_offset(&mut self, offset: u64) -> Option<ByteBuffer> {
        self.file
            .seek(SeekFrom::Start(offset))
            .expect("Unable to find offset in file.");

        let file_info =
            FileInfo::read_options(&mut self.file, self.platform.endianness(), ()).ok()?;

        match file_info.file_type {
            FileType::Empty => None,
            FileType::Standard => self.read_standard_file(offset, &file_info),
            FileType::Model => self.read_model_file(offset, &file_info),
            FileType::Texture => self.read_texture_file(offset, &file_info),
        }
    }

    /// Reads a standard file block.
    fn read_standard_file(&mut self, offset: u64, file_info: &FileInfo) -> Option<ByteBuffer> {
        let standard_file_info = file_info.standard_info.as_ref()?;

        let mut blocks: Vec<Block> = Vec::with_capacity(standard_file_info.num_blocks as usize);

        for _ in 0..standard_file_info.num_blocks {
            blocks.push(Block::read_options(&mut self.file, self.platform.endianness(), ()).ok()?);
        }

        let mut data: Vec<u8> = Vec::with_capacity(file_info.file_size as usize);

        let starting_position = offset + (file_info.size as u64);

        for i in 0..standard_file_info.num_blocks {
            data.append(
                &mut read_data_block(
                    &mut self.file,
                    self.platform.endianness(),
                    starting_position + (blocks[i as usize].offset as u64),
                )
                .expect("Failed to read data block."),
            );
        }

        Some(data)
    }

    /// Reads a model file block.
    fn read_model_file(&mut self, offset: u64, file_info: &FileInfo) -> Option<ByteBuffer> {
        let model_file_info = file_info.model_info.as_ref()?;

        let mut buffer = Cursor::new(Vec::new());

        let base_offset = offset + (file_info.size as u64);

        let total_blocks = model_file_info.num.total();

        let compressed_block_sizes: Vec<u16> = self
            .file
            .read_type_args(
                self.platform.endianness(),
                VecArgs::builder().count(total_blocks as usize).finalize(),
            )
            .ok()?;

        let mut current_block = 0;

        let mut vertex_data_offsets: [u32; 3] = [0; 3];
        let mut vertex_data_sizes: [u32; 3] = [0; 3];

        let mut index_data_offsets: [u32; 3] = [0; 3];
        let mut index_data_sizes: [u32; 3] = [0; 3];

        // start writing at 0x44
        buffer.seek(SeekFrom::Start(0x44)).ok()?;

        self.file
            .seek(SeekFrom::Start(
                base_offset + (model_file_info.offset.stack_size as u64),
            ))
            .ok()?;

        // read from stack blocks
        let mut read_model_blocks = |offset: u64, size: usize| -> Option<u64> {
            self.file.seek(SeekFrom::Start(base_offset + offset)).ok()?;
            let stack_start = buffer.position();
            for _ in 0..size {
                let last_pos = &self.file.stream_position().ok()?;

                let data = read_data_block(&self.file, self.platform.endianness(), *last_pos)
                    .expect("Unable to read block data.");
                // write to buffer
                buffer.write_all(data.as_slice()).ok()?;

                self.file
                    .seek(SeekFrom::Start(
                        last_pos + (compressed_block_sizes[current_block] as u64),
                    ))
                    .ok()?;
                current_block += 1;
            }

            Some(buffer.position() - stack_start)
        };

        let stack_size = read_model_blocks(
            model_file_info.offset.stack_size as u64,
            model_file_info.num.stack_size as usize,
        )? as u32;
        let runtime_size = read_model_blocks(
            model_file_info.offset.runtime_size as u64,
            model_file_info.num.runtime_size as usize,
        )? as u32;

        let mut process_model_data =
            |i: usize,
             size: u32,
             offset: u32,
             offsets: &mut [u32; 3],
             data_sizes: &mut [u32; 3]| {
                if size != 0 {
                    let current_vertex_offset = buffer.position() as u32;
                    if i == 0 || current_vertex_offset != offsets[i - 1] {
                        offsets[i] = current_vertex_offset;
                    } else {
                        offsets[i] = 0;
                    }

                    self.file
                        .seek(SeekFrom::Start(base_offset + (offset as u64)))
                        .ok();

                    for _ in 0..size {
                        let last_pos = self.file.stream_position().unwrap();

                        let data =
                            read_data_block(&self.file, self.platform.endianness(), last_pos)
                                .expect("Unable to read raw model block!");

                        buffer
                            .write_all(data.as_slice())
                            .expect("Unable to write to memory buffer!");

                        data_sizes[i] += data.len() as u32;
                        self.file
                            .seek(SeekFrom::Start(
                                last_pos + (compressed_block_sizes[current_block] as u64),
                            ))
                            .expect("Unable to seek properly.");
                        current_block += 1;
                    }
                }
            };

        // process all 3 lods
        for i in 0..3 {
            // process vertices
            process_model_data(
                i,
                model_file_info.num.vertex_buffer_size[i] as u32,
                model_file_info.offset.vertex_buffer_size[i],
                &mut vertex_data_offsets,
                &mut vertex_data_sizes,
            );

            // TODO: process edges

            // process indices
            process_model_data(
                i,
                model_file_info.num.index_buffer_size[i] as u32,
                model_file_info.offset.index_buffer_size[i],
                &mut index_data_offsets,
                &mut index_data_sizes,
            );
        }

        let header = ModelFileHeader {
            version: model_file_info.version,
            stack_size,
            runtime_size,
            vertex_declaration_count: model_file_info.vertex_declaration_num,
            material_count: model_file_info.material_num,
            vertex_offsets: vertex_data_offsets,
            index_offsets: index_data_offsets,
            vertex_buffer_size: vertex_data_sizes,
            index_buffer_size: index_data_sizes,
            lod_count: model_file_info.num_lods,
            index_buffer_streaming_enabled: model_file_info.index_buffer_streaming_enabled,
            has_edge_geometry: model_file_info.edge_geometry_enabled,
        };

        buffer.seek(SeekFrom::Start(0)).ok()?;

        header
            .write_options(&mut buffer, self.platform.endianness(), ())
            .ok()?;

        Some(buffer.into_inner())
    }

    /// Reads a texture file block.
    fn read_texture_file(&mut self, offset: u64, file_info: &FileInfo) -> Option<ByteBuffer> {
        let texture_file_info = file_info.texture_info.as_ref()?;

        let mut data: Vec<u8> = Vec::with_capacity(file_info.file_size as usize);

        // write the header if it exists
        let mipmap_size = texture_file_info.lods[0].compressed_size;
        if mipmap_size != 0 {
            let original_pos = self.file.stream_position().ok()?;

            self.file
                .seek(SeekFrom::Start(offset + file_info.size as u64))
                .ok()?;

            let mut header = vec![0u8; texture_file_info.lods[0].compressed_offset as usize];
            self.file.read_exact(&mut header).ok()?;

            data.append(&mut header);

            self.file.seek(SeekFrom::Start(original_pos)).ok()?;
        }

        for i in 0..texture_file_info.num_blocks {
            let mut running_block_total = (texture_file_info.lods[i as usize].compressed_offset
                as u64)
                + offset
                + (file_info.size as u64);

            for _ in 0..texture_file_info.lods[i as usize].block_count {
                let original_pos = self.file.stream_position().ok()?;

                data.append(&mut read_data_block(
                    &self.file,
                    self.platform.endianness(),
                    running_block_total,
                )?);

                self.file.seek(SeekFrom::Start(original_pos)).ok()?;

                running_block_total += self.file.read_le::<i16>().ok()? as u64;
            }
        }

        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_invalid() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("random");

        let mut dat = SqPackData::from_existing(Platform::Win32, d.to_str().unwrap()).unwrap();

        let empty_file_info = FileInfo {
            size: 0,
            file_type: FileType::Empty,
            file_size: 0,
            standard_info: None,
            model_info: None,
            texture_info: None,
        };

        // Reading invalid data should just be nothing, but no panics
        assert!(dat.read_from_offset(0).is_none());
        assert!(dat.read_standard_file(0, &empty_file_info).is_none());
        assert!(dat.read_model_file(0, &empty_file_info).is_none());
        assert!(dat.read_texture_file(0, &empty_file_info).is_none());
    }
}
