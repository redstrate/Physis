use std::io::{Cursor, Read, Seek, SeekFrom};
use binrw::BinRead;
use binrw::binrw;
use crate::gamedata::MemoryBuffer;
use crate::model::ModelFileHeader;
use std::io::Write;
use binrw::BinWrite;
use crate::sqpack::read_data_block;

#[binrw]
#[brw(repr = i32)]
#[derive(Debug, PartialEq)]
pub enum FileType {
    Empty = 1,
    Standard,
    Model,
    Texture,
}

#[derive(BinRead)]
struct StandardFileBlock {
    #[br(pad_before = 8)]
    num_blocks: u32,
}

#[derive(BinRead)]
struct TextureLodBlock {
    compressed_offset: u32,
    compressed_size: u32,
    decompressed_size: u32,

    block_offset: u32,
    block_count: u32,
}

#[binrw]
pub struct ModelMemorySizes<T: 'static + binrw::BinRead<Args=()> + binrw::BinWrite<Args=()> + Default + std::ops::AddAssign + Copy> {
    pub stack_size: T,
    pub runtime_size: T,

    pub vertex_buffer_size: [T; 3],
    pub edge_geometry_vertex_buffer_size: [T; 3],
    pub index_buffer_size: [T; 3],
}

impl<T: 'static + binrw::BinRead<Args=()> + binrw::BinWrite<Args=()> + Default + std::ops::AddAssign + Copy> ModelMemorySizes<T> {
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

#[derive(BinRead)]
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

    #[br(map = | x: u8 | x != 0)]
    pub index_buffer_streaming_enabled: bool,
    #[brw(pad_after = 1)]
    #[br(map = | x: u8 | x != 0)]
    pub edge_geometry_enabled: bool,
}

#[derive(BinRead)]
struct TextureBlock {
    #[br(pad_before = 8)]
    num_blocks: u32,

    #[br(count = num_blocks)]
    lods: Vec<TextureLodBlock>,
}

/// A SqPack file info header. It can optionally contain extra information, such as texture or
/// model data depending on the file type.
#[derive(BinRead)]
struct FileInfo {
    size: u32,
    file_type: FileType,
    file_size: i32,

    #[br(if (file_type == FileType::Standard))]
    standard_info: Option<StandardFileBlock>,

    #[br(if (file_type == FileType::Model))]
    model_info: Option<ModelFileBlock>,

    #[br(if (file_type == FileType::Texture))]
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
    Compressed { compressed_length: i32, decompressed_length: i32 },
    Uncompressed { file_size: i32 },
}

#[binrw::binread]
#[derive(Debug)]
pub struct BlockHeader {
    #[br(pad_after = 4)]
    pub size : u32,

    #[br(temp)]
    x : i32,

    #[br(temp)]
    y : i32,

    #[br(args { x, y })]
    #[br(restore_position)]
    pub compression: CompressionMode,
}

pub struct DatFile {
    file: std::fs::File,
}

// from https://users.rust-lang.org/t/how-best-to-convert-u8-to-u16/57551/4
fn to_u8_slice(slice: &mut [u16]) -> &mut [u8] {
    let byte_len = 2 * slice.len();
    unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_mut_ptr().cast::<u8>(),
            byte_len,
        )
    }
}

impl DatFile {
    /// Creates a new reference to an existing dat file.
    pub fn from_existing(path: &str) -> Option<DatFile> {
        Some(DatFile {
            file: std::fs::File::open(path).ok()?
        })
    }

    pub fn read_from_offset(&mut self, offset: u32) -> Option<MemoryBuffer> {
        let offset: u64 = (offset * 0x80) as u64;

        self.file.seek(SeekFrom::Start(offset))
            .expect("TODO: panic message");

        let file_info = FileInfo::read(&mut self.file)
            .expect("Failed to parse file info.");

        match file_info.file_type {
            FileType::Empty => None,
            FileType::Standard => {
                let standard_file_info = file_info.standard_info.unwrap();

                let mut blocks: Vec<Block> = Vec::with_capacity(standard_file_info.num_blocks as usize);

                for _ in 0..standard_file_info.num_blocks {
                    blocks.push(Block::read(&mut self.file).unwrap());
                }

                let mut data: Vec<u8> = Vec::with_capacity(file_info.file_size as usize);

                let starting_position = offset + (file_info.size as u64);

                for i in 0..standard_file_info.num_blocks {
                    data.append(&mut read_data_block(&mut self.file, starting_position + (blocks[i as usize].offset as u64))
                        .expect("Failed to read data block."));
                }

                Some(data)
            }
            FileType::Model => {
                let mut buffer = Cursor::new(Vec::new());

                let model_file_info = file_info.model_info.unwrap();

                let base_offset = offset + (file_info.size as u64);

                let total_blocks = model_file_info.num.total();

                let mut compressed_block_sizes: Vec<u16> = vec![0; total_blocks as usize];
                let slice: &mut [u8] = to_u8_slice(&mut compressed_block_sizes);

                self.file.read_exact(slice).ok()?;

                let mut current_block = 0;

                let mut vertex_data_offsets: [u32; 3] = [0; 3];
                let mut vertex_data_sizes: [u32; 3] = [0; 3];

                let mut index_data_offsets: [u32; 3] = [0; 3];
                let mut index_data_sizes: [u32; 3] = [0; 3];

                // start writing at 0x44
                buffer.seek(SeekFrom::Start(0x44)).ok()?;

                self.file.seek(SeekFrom::Start(base_offset + (model_file_info.offset.stack_size as u64))).ok()?;

                // read from stack blocks
                let mut read_model_blocks = |offset: u64, size: usize| -> Option<u64> {
                    self.file.seek(SeekFrom::Start(base_offset + offset)).ok()?;
                    let stack_start = buffer.position();
                    for _ in 0..size {
                        let last_pos = &self.file.stream_position().unwrap();

                        let data = read_data_block(&self.file, *last_pos)
                            .expect("Unable to read block data.");
                        // write to buffer
                        buffer.write(data.as_slice()).ok()?;

                        self.file.seek(SeekFrom::Start(last_pos + (compressed_block_sizes[current_block as usize] as u64))).ok()?;
                        current_block += 1;
                    }

                    Some(buffer.position() - stack_start)
                };

                let stack_size = read_model_blocks(model_file_info.offset.stack_size as u64, model_file_info.num.stack_size as usize).unwrap() as u32;
                let runtime_size = read_model_blocks(model_file_info.offset.runtime_size as u64, model_file_info.num.runtime_size as usize).unwrap() as u32;

                let mut process_model_data = |i: usize, size: u32, offset: u32, offsets: &mut [u32; 3], data_sizes: &mut [u32; 3]| {
                    if size != 0 {
                        let current_vertex_offset = buffer.position() as u32;
                        if i == 0 || current_vertex_offset != offsets[i - 1] {
                            offsets[i] = current_vertex_offset;
                        } else {
                            offsets[i] = 0;
                        }

                        self.file.seek(SeekFrom::Start(base_offset + (offset as u64))).ok();

                        for _ in 0..size {
                            let last_pos = self.file.stream_position().unwrap();

                            let data = read_data_block(&self.file, last_pos)
                                .expect("Unable to read raw model block!");

                            buffer.write(data.as_slice()).expect("Unable to write to memory buffer!");

                            data_sizes[i] += data.len() as u32;
                            self.file.seek(SeekFrom::Start(last_pos + (compressed_block_sizes[current_block] as u64)))
                                .expect("Unable to seek properly.");
                            current_block += 1;
                        }
                    }
                };

                // process all 3 lods
                for i in 0..3 {
                    // process vertices
                    process_model_data(i, model_file_info.num.vertex_buffer_size[i] as u32, model_file_info.offset.vertex_buffer_size[i], &mut vertex_data_offsets, &mut vertex_data_sizes);

                    // TODO: process edges

                    // process indices
                    process_model_data(i, model_file_info.num.index_buffer_size[i] as u32, model_file_info.offset.index_buffer_size[i], &mut index_data_offsets, &mut index_data_sizes);
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

                header.write_to(&mut buffer).ok()?;

                Some(buffer.into_inner())
            }
            FileType::Texture => {
                let mut data: Vec<u8> = Vec::with_capacity(file_info.file_size as usize);

                let texture_file_info = file_info.texture_info.unwrap();

                for i in 0..texture_file_info.num_blocks {
                    let mut running_block_total = (texture_file_info.lods[i as usize].compressed_offset as u64) + offset + (file_info.size as u64);

                    data.append(&mut read_data_block(&self.file, running_block_total).unwrap());

                    for _ in 1..texture_file_info.lods[i as usize].block_count {
                        running_block_total += i16::read(&mut self.file).unwrap() as u64;
                        data.append(&mut read_data_block(&self.file, running_block_total).unwrap());
                    }

                    // dummy?
                    i16::read(&mut self.file).unwrap();
                }

                Some(data)
            }
        }
    }
}