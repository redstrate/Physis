use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Cursor, Seek, SeekFrom, Write};
use binrw::BinRead;
use binrw::binread;
use crate::{BootData, GameData};
use crate::sqpack::{read_data_block, read_data_block_patch};
use core::cmp::min;
use crate::patch::TargetHeaderKind::Version;

#[derive(BinRead, Debug)]
struct PatchChunk {
    #[br(big)]
    size: u32,
    chunk_type: ChunkType,
    #[br(if(chunk_type != ChunkType::EndOfFile))]
    crc32: u32
}

#[derive(BinRead, PartialEq, Debug)]
enum ChunkType {
    #[br(magic = b"FHDR")] FileHeader(FileHeaderChunk),
    #[br(magic = b"APLY")] ApplyOption(ApplyOptionChunk),
    #[br(magic = b"ADIR")] AddDirectory(AddDirectoryChunk),
    #[br(magic = b"DELD")] DeleteDirectory(DeleteDirectoryChunk),
    #[br(magic = b"SQPK")] Sqpk(SqpkChunk),
    #[br(magic = b"EOF_")] EndOfFile,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct FileHeaderChunk {
    #[br(pad_before = 2)]
    #[br(pad_after = 1)]
    version: u8,

    #[br(count = 4)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    name: String,

    #[br(big)]
    entry_files: u32,

    #[br(big)]
    add_directories : u32,

    #[br(big)]
    delete_directories : u32,

    #[br(big)]
    delete_data_size : u32,

    #[br(big)]
    delete_data_size_2 : u32,

    #[br(big)]
    minor_version : u32,

    #[br(big)]
    repository_name : u32,

    #[br(big)]
    commands : u32,

    #[br(big)]
    sqpk_add_commands: u32,

    #[br(big)]
    sqpk_delete_commands: u32,

    #[br(big)]
    sqpk_expand_commands: u32,

    #[br(big)]
    sqpk_header_commands: u32,

    #[br(big)]
    #[br(pad_after = 0xB8)]
    sqpk_file_commands: u32
}

#[binrw::binread]
#[derive(PartialEq, Debug)]
struct ApplyOptionChunk {
    #[br(pad_after = 4)]
    option: u32,
    value: u32,
}

#[binrw::binread]
#[derive(PartialEq, Debug)]
struct AddDirectoryChunk {
    #[br(temp)]
    path_length: u32,

    #[br(count = path_length)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    name: String,
}

#[binread]
#[br(big)]
#[derive(PartialEq, Debug)]
struct DeleteDirectoryChunk {
    #[br(temp)]
    path_length: u32,

    #[br(count = path_length)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    name: String,
}

#[binread]
#[derive(PartialEq, Debug)]
enum SqpkOperation {
    #[br(magic = b'A')] AddData(SqpkAddData),
    #[br(magic = b'D')] DeleteData(SqpkDeleteData),
    #[br(magic = b'E')] ExpandData(SqpkDeleteData),
    #[br(magic = b'F')] FileOperation(SqpkFileOperationData),
    #[br(magic = b'H')] HeaderUpdate(SqpkHeaderUpdateData),
    #[br(magic = b'I')] IndexAddDelete(SqpkIndexData),
    #[br(magic = b'X')] PatchInfo(SqpkPatchInfo),
    #[br(magic = b'T')] TargetInfo(SqpkTargetInfo),
}

#[derive(BinRead, PartialEq, Debug)]
enum SqpkIndexCommand {
    #[br(magic = b'A')] Add,
    #[br(magic = b'D')] Delete
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkIndexData {
    command : SqpkIndexCommand,

    #[br(map = | x: u8 | x != 0)]
    #[br(pad_after = 1)]
    is_synonym : bool,

    main_id : u16,
    sub_id : u16,
    file_id : u32,

    file_hash : u64,

    block_offset : u32,
    block_number : u32
}

#[derive(BinRead, PartialEq, Debug)]
struct SqpkPatchInfo {
    status: u8,
    #[br(pad_after = 1)]
    version: u8,

    #[br(big)]
    install_size : u64
}

#[binread]
#[derive(PartialEq, Debug)]
enum SqpkFileOperation {
    #[br(magic = b'A')] AddFile,
    #[br(magic = b'R')] RemoveAll,
    #[br(magic = b'D')] DeleteFile
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkAddData {
    #[br(pad_before = 3)]
    main_id : u16,
    sub_id : u16,
    file_id : u32,

    #[br(map = | x : i32 | x << 7 )]
    block_offset : i32,
    #[br(map = | x : i32 | x << 7 )]
    block_number : i32,
    #[br(map = | x : i32 | x << 7 )]
    block_delete_number : i32,

    #[br(count = block_number)]
    block_data : Vec<u8>
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkDeleteData {
    #[br(pad_before = 3)]
    main_id : u16,
    sub_id : u16,
    file_id : u32,

    #[br(map = | x : i32 | x << 7 )]
    block_offset : i32,
    #[br(pad_after = 4)]
    block_number : i32
}

#[binread]
#[derive(PartialEq, Debug)]
enum TargetFileKind {
    #[br(magic=b'D')] Dat,
    #[br(magic=b'I')] Index
}

#[binread]
#[derive(PartialEq, Debug)]
enum TargetHeaderKind {
    #[br(magic=b'V')] Version,
    #[br(magic=b'I')] Index,
    #[br(magic=b'D')] Data
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkHeaderUpdateData {
    file_kind : TargetFileKind,
    header_kind : TargetHeaderKind,

    #[br(pad_before = 1)]
    main_id : u16,
    sub_id : u16,
    file_id : u32,

    #[br(count = 1024)]
    header_data : Vec<u8>
}

#[binread]
#[derive(PartialEq, Debug)]
#[br(big)]
struct SqpkFileOperationData {
    #[br(pad_after = 2)]
    operation: SqpkFileOperation,

    offset : i64,
    file_size : u64,
    #[br(temp)]
    path_length : u32,
    expansion_id : u32,

    #[br(count = path_length)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x[..x.len() - 1].to_vec()).unwrap())]
    path: String,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkTargetInfo {
    #[br(pad_before = 3)]
    platform : u16,
    region : i16,
    is_debug : i16,
    version : u16,
    #[br(little)]
    deleted_data_size : u64,
    #[br(little)]
    #[br(pad_after = 96)]
    seek_count : u64
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkChunk {
    size: u32,
    operation : SqpkOperation
}

const WIPE_BUFFER: [u8; 1 << 16] = [0; 1 << 16];

fn wipe(mut file : &File, length : i32) {
    let mut length = length;
    while length > 0 {
        let num_bytes = min(WIPE_BUFFER.len() as i32, length);
        file.write(&WIPE_BUFFER[0..num_bytes as usize]);
        length -= num_bytes;
    }
}

fn wipe_from_offset(mut file : &File, length : i32, offset : i32) {
    file.seek(SeekFrom::Start(offset as u64));
    wipe(file, length);
}

fn write_empty_file_block_at(mut file : &File, offset : i32, block_number : i32) {
    wipe_from_offset(file, block_number << 7, offset);

    file.seek(SeekFrom::Start(offset as u64));

    let block_size : i32 = 1 << 7;
    file.write(block_size.to_le_bytes().as_slice());

    let unknown : i32 = 0;
    file.write(unknown.to_le_bytes().as_slice());

    let file_size : i32 = 0;
    file.write(file_size.to_le_bytes().as_slice());

    let num_blocks : i32 = block_number - 1;
    file.write(num_blocks.to_le_bytes().as_slice());

    let used_blocks : i32 = 0;
    file.write(used_blocks.to_le_bytes().as_slice());
}

pub fn process_patch(data_dir : &str, path : &str) {
    let mut file = File::open(path).unwrap();

    file.seek(SeekFrom::Start(12));

    loop {
        let chunk = PatchChunk::read(&mut file).unwrap();

        match chunk.chunk_type {
            ChunkType::Sqpk(pchunk) => {
                match pchunk.operation {
                    SqpkOperation::AddData(add) => {
                        let filename = format!("{}/sqpack/ffxiv/{:02x}{:04x}.win32.dat{}", data_dir, add.main_id, add.sub_id, add.file_id);

                        let mut new_file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(filename).unwrap();

                        new_file.seek(SeekFrom::Start(add.block_offset as u64));

                        new_file.write(&*add.block_data);

                        wipe(&mut new_file, add.block_delete_number);
                    }
                    SqpkOperation::DeleteData(delete) => {
                        let filename = format!("{}/sqpack/ffxiv/{:02x}{:04x}.win32.dat{}", data_dir, delete.main_id, delete.sub_id, delete.file_id);

                        let mut new_file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(filename).unwrap();

                        write_empty_file_block_at(&mut new_file, delete.block_offset, delete.block_number);
                    }
                    SqpkOperation::ExpandData(expand) => {
                        let filename = format!("{}/sqpack/ffxiv/{:02x}{:04x}.win32.dat{}", data_dir, expand.main_id, expand.sub_id, expand.file_id);

                        let mut new_file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(filename).unwrap();

                        write_empty_file_block_at(&mut new_file, expand.block_offset, expand.block_number);
                    }
                    SqpkOperation::HeaderUpdate(header) => {
                        let mut file_path : String;

                        match header.file_kind {
                            TargetFileKind::Dat => {
                                file_path = format!("{}/sqpack/ffxiv/{:02x}{:04x}.win32.dat{}", data_dir, header.main_id, header.sub_id, header.file_id);
                            }
                            TargetFileKind::Index => {
                                file_path = format!("{}/sqpack/ffxiv/{:02x}{:04x}.win32.index", data_dir, header.main_id, header.sub_id);

                                // index files have no special ending if it's file_id == 0
                                if header.file_id != 0 {
                                    file_path += &*format!("{}", header.file_id);
                                }
                            }
                        }

                        let mut new_file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(file_path.as_str()).unwrap();

                        if header.header_kind != Version {
                            new_file.seek(SeekFrom::Start(1024));
                        }

                        new_file.write(&*header.header_data);
                    }
                    SqpkOperation::FileOperation(fop) => {
                        match fop.operation {
                            SqpkFileOperation::AddFile => {
                                let new_path = data_dir.to_owned() + "/" + &fop.path;

                                let (left, right) = new_path.rsplit_once('/').unwrap();

                                fs::create_dir_all(left);

                                // reverse reading crc32
                                file.seek(SeekFrom::Current(-4));

                                let mut data: Vec<u8> = Vec::with_capacity(fop.file_size as usize);

                                while data.len() < fop.file_size as usize {
                                    data.append(&mut read_data_block_patch(&mut file).unwrap());
                                }

                                // re-apply crc32
                                file.seek(SeekFrom::Current(4));

                                // now apply the file!
                                let mut new_file = OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .open(new_path).unwrap();
                                new_file.seek(SeekFrom::Start(fop.offset as u64));

                                new_file.write(&mut data);
                            }
                            SqpkFileOperation::DeleteFile => {
                                let new_path = data_dir.to_owned() + "/" + &fop.path;

                                fs::remove_file(new_path.as_str());
                            }
                            _ => {
                                panic!("Unhandled operation!");
                            }
                        }
                    }
                    _ => {}
                }
            }
            ChunkType::EndOfFile => {
                return;
            }
            _ => {}
        }
    }
}