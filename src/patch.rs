// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use core::cmp::min;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;

use binrw::binread;
use binrw::BinRead;
use binrw::binrw;
use tracing::{debug, warn};

use crate::common::Region;
use crate::sqpack::read_data_block_patch;

#[binread]
#[derive(Debug)]
#[br(little)]
struct PatchHeader {
    #[br(temp)]
    #[br(count = 7)]
    #[br(pad_before = 1)]
    #[br(pad_after = 4)]
    #[br(assert(magic == b"ZIPATCH"))]
    magic: Vec<u8>,
}

#[derive(BinRead, Debug)]
#[allow(dead_code)]
#[br(little)]
struct PatchChunk {
    #[br(big)]
    size: u32,
    chunk_type: ChunkType,
    #[br(if(chunk_type != ChunkType::EndOfFile))]
    crc32: u32,
}

#[derive(BinRead, PartialEq, Debug)]
enum ChunkType {
    #[br(magic = b"FHDR")]
    FileHeader(
        #[br(pad_before = 2)]
        #[br(pad_after = 1)]
        FileHeaderChunk,
    ),
    #[br(magic = b"APLY")]
    ApplyOption(ApplyOptionChunk),
    #[br(magic = b"ADIR")]
    AddDirectory(DirectoryChunk),
    #[br(magic = b"DELD")]
    DeleteDirectory(DirectoryChunk),
    #[br(magic = b"SQPK")]
    Sqpk(SqpkChunk),
    #[br(magic = b"EOF_")]
    EndOfFile,
}

#[derive(BinRead, PartialEq, Debug)]
enum FileHeaderChunk {
    #[br(magic = 2u8)]
    Version2(FileHeaderChunk2),
    #[br(magic = 3u8)]
    Version3(FileHeaderChunk3),
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct FileHeaderChunk2 {
    #[br(count = 4)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    name: String,

    #[br(pad_before = 8)]
    depot_hash: u32,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct FileHeaderChunk3 {
    #[br(count = 4)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    name: String,

    entry_files: u32,

    add_directories: u32,
    delete_directories: u32,
    delete_data_size: u32,
    delete_data_size_2: u32,
    minor_version: u32,
    repository_name: u32,
    commands: u32,
    sqpk_add_commands: u32,
    sqpk_delete_commands: u32,
    sqpk_expand_commands: u32,
    sqpk_header_commands: u32,
    #[br(pad_after = 0xB8)]
    sqpk_file_commands: u32,
}

#[binread]
#[br(repr = u32)]
#[br(big)]
#[derive(PartialEq, Debug)]
enum ApplyOption {
    IgnoreMissing = 1,
    IgnoreOldMismatch = 2,
}

#[binrw::binread]
#[derive(PartialEq, Debug)]
struct ApplyOptionChunk {
    #[br(pad_after = 4)]
    option: ApplyOption,
    #[br(big)]
    value: u32,
}

#[binrw::binread]
#[derive(PartialEq, Debug)]
struct DirectoryChunk {
    #[br(temp)]
    path_length: u32,

    #[br(count = path_length)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x).unwrap())]
    name: String,
}

#[binread]
#[derive(PartialEq, Debug)]
enum SqpkOperation {
    #[br(magic = b'A')]
    AddData(SqpkAddData),
    #[br(magic = b'D')]
    DeleteData(SqpkDeleteData),
    #[br(magic = b'E')]
    ExpandData(SqpkDeleteData),
    #[br(magic = b'F')]
    FileOperation(SqpkFileOperationData),
    #[br(magic = b'H')]
    HeaderUpdate(SqpkHeaderUpdateData),
    #[br(magic = b'X')]
    PatchInfo(SqpkPatchInfo),
    #[br(magic = b'T')]
    TargetInfo(SqpkTargetInfo),
    #[br(magic = b'I')]
    Index(SqpkIndex)
}

#[derive(BinRead, PartialEq, Debug)]
struct SqpkPatchInfo {
    status: u8,
    #[br(pad_after = 1)]
    version: u8,

    #[br(big)]
    install_size: u64,
}

#[binread]
#[derive(PartialEq, Debug)]
enum SqpkFileOperation {
    #[br(magic = b'A')]
    AddFile,
    #[br(magic = b'R')]
    RemoveAll,
    #[br(magic = b'D')]
    DeleteFile,
    #[br(magic = b'M')]
    MakeDirTree,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkAddData {
    #[br(pad_before = 3)]
    main_id: u16,
    sub_id: u16,
    file_id: u32,

    #[br(map = | x : u32 | x << 7 )]
    block_offset: u32,
    #[br(map = | x : u32 | x << 7 )]
    block_number: u32,
    #[br(map = | x : u32 | x << 7 )]
    block_delete_number: u32,

    #[br(count = block_number)]
    block_data: Vec<u8>,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkDeleteData {
    #[br(pad_before = 3)]
    main_id: u16,
    sub_id: u16,
    file_id: u32,

    #[br(map = | x : u32 | x << 7 )]
    block_offset: u32,
    #[br(pad_after = 4)]
    block_number: u32,
}

#[binread]
#[derive(PartialEq, Debug)]
enum TargetFileKind {
    #[br(magic = b'D')]
    Dat,
    #[br(magic = b'I')]
    Index,
}

#[binread]
#[derive(PartialEq, Debug)]
enum TargetHeaderKind {
    #[br(magic = b'V')]
    Version,
    #[br(magic = b'I')]
    Index,
    #[br(magic = b'D')]
    Data,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkHeaderUpdateData {
    file_kind: TargetFileKind,
    header_kind: TargetHeaderKind,

    #[br(pad_before = 1)]
    main_id: u16,
    sub_id: u16,
    file_id: u32,

    #[br(count = 1024)]
    header_data: Vec<u8>,
}

#[binread]
#[derive(PartialEq, Debug)]
#[br(big)]
struct SqpkFileOperationData {
    #[br(pad_after = 2)]
    operation: SqpkFileOperation,

    offset: u64,
    file_size: u64,

    #[br(temp)]
    path_length: u32,

    #[br(pad_after = 2)]
    expansion_id: u16,

    #[br(count = path_length)]
    #[br(map = | x: Vec < u8 > | String::from_utf8(x[..x.len() - 1].to_vec()).unwrap())]
    path: String,
}

#[binrw]
#[brw(repr = u16)]
#[derive(Debug, PartialEq)]
enum PlatformId {
    Windows,
    PS3,
    PS4,
}

fn get_platform_string(id: &PlatformId) -> &'static str {
    match &id {
        PlatformId::Windows => "win32",
        PlatformId::PS3 => "ps3.d",
        PlatformId::PS4 => "ps4.d",
    }
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkTargetInfo {
    #[br(pad_before = 3)]
    platform: PlatformId,
    region: Region,
    #[br(map = | x : u16 | x == 1)]
    is_debug: bool,
    version: u16,
    #[br(little)]
    deleted_data_size: u64,
    #[br(little)]
    #[br(pad_after = 96)]
    seek_count: u64,
}

#[binread]
#[derive(PartialEq, Debug)]
enum SqpkIndexCommand {
    #[br(magic = b'A')]
    Add,
    #[br(magic = b'D')]
    Delete,
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkIndex {
    command: SqpkIndexCommand,
    #[br(map = | x : u8 | x == 1)]
    is_synonym: bool,

    #[br(pad_before = 1)]
    file_hash: u64,

    block_offset: u32,
    #[br(pad_after = 8)] // data?
    block_number: u32
}

#[derive(BinRead, PartialEq, Debug)]
#[br(big)]
struct SqpkChunk {
    size: u32,
    operation: SqpkOperation,
}

const WIPE_BUFFER: [u8; 1 << 16] = [0; 1 << 16];

fn wipe(mut file: &File, length: u32) -> Result<(), PatchError> {
    let mut length: usize = length as usize;
    while length > 0 {
        let num_bytes = min(WIPE_BUFFER.len(), length);
        file.write_all(&WIPE_BUFFER[0..num_bytes])?;
        length -= num_bytes;
    }

    Ok(())
}

fn wipe_from_offset(mut file: &File, length: u32, offset: u32) -> Result<(), PatchError> {
    file.seek(SeekFrom::Start(offset as u64))?;
    wipe(file, length)
}

fn write_empty_file_block_at(
    mut file: &File,
    offset: u32,
    block_number: u32,
) -> Result<(), PatchError> {
    wipe_from_offset(file, block_number << 7, offset)?;

    file.seek(SeekFrom::Start(offset as u64))?;

    let block_size: i32 = 1 << 7;
    file.write_all(block_size.to_le_bytes().as_slice())?;

    let unknown: i32 = 0;
    file.write_all(unknown.to_le_bytes().as_slice())?;

    let file_size: i32 = 0;
    file.write_all(file_size.to_le_bytes().as_slice())?;

    let num_blocks: i32 = (block_number - 1).try_into().unwrap();
    file.write_all(num_blocks.to_le_bytes().as_slice())?;

    let used_blocks: i32 = 0;
    file.write_all(used_blocks.to_le_bytes().as_slice())?;

    Ok(())
}

fn get_expansion_folder_sub(sub_id: u16) -> String {
    let expansion_id = sub_id >> 8;

    get_expansion_folder(expansion_id)
}

fn get_expansion_folder(id: u16) -> String {
    match id {
        0 => "ffxiv".to_string(),
        n => format!("ex{}", n),
    }
}

#[derive(Debug)]
/// Errors emitted in the patching process
pub enum PatchError {
    /// Failed to read parts of the file
    InvalidPatchFile,
    /// Failed to parse the patch format
    ParseError,
}

impl From<std::io::Error> for PatchError {
    // TODO: implement specific PatchErrors for stuff like out of storage space. invalidpatchfile is a bad name for this
    fn from(_: std::io::Error) -> Self {
        PatchError::InvalidPatchFile
    }
}

impl From<binrw::Error> for PatchError {
    fn from(_: binrw::Error) -> Self {
        PatchError::ParseError
    }
}

/// Applies a boot or a game patch to the specified _data_dir_.
pub fn apply_patch(data_dir: &str, patch_path: &str) -> Result<(), PatchError> {
    let mut file = File::open(patch_path)?;

    PatchHeader::read(&mut file)?;

    let mut target_info: Option<SqpkTargetInfo> = None;

    let get_dat_path =
        |target_info: &SqpkTargetInfo, main_id: u16, sub_id: u16, file_id: u32| -> String {
            let filename = format!(
                "{:02x}{:04x}.{}.dat{}",
                main_id,
                sub_id,
                get_platform_string(&target_info.platform),
                file_id
            );
            let path: PathBuf = [
                data_dir,
                "sqpack",
                &get_expansion_folder_sub(sub_id),
                &filename,
            ]
            .iter()
            .collect();

            path.to_str().unwrap().to_string()
        };

    let get_index_path =
        |target_info: &SqpkTargetInfo, main_id: u16, sub_id: u16, file_id: u32| -> String {
            let mut filename = format!(
                "{:02x}{:04x}.{}.index",
                main_id,
                sub_id,
                get_platform_string(&target_info.platform)
            );

            // index files have no special ending if it's file_id == 0
            if file_id != 0 {
                filename += &*format!("{}", file_id);
            }

            let path: PathBuf = [
                data_dir,
                "sqpack",
                &get_expansion_folder_sub(sub_id),
                &filename,
            ]
            .iter()
            .collect();

            path.to_str().unwrap().to_string()
        };

    loop {
        let chunk = PatchChunk::read(&mut file)?;

        match chunk.chunk_type {
            ChunkType::Sqpk(pchunk) => {
                match pchunk.operation {
                    SqpkOperation::AddData(add) => {
                        let filename = get_dat_path(
                            target_info.as_ref().unwrap(),
                            add.main_id,
                            add.sub_id,
                            add.file_id,
                        );

                        let (left, _) = filename.rsplit_once('/').unwrap();
                        fs::create_dir_all(left)?;

                        let mut new_file =
                            OpenOptions::new().write(true).create(true).open(filename)?;

                        new_file.seek(SeekFrom::Start(add.block_offset as u64))?;

                        new_file.write_all(&add.block_data)?;

                        wipe(&new_file, add.block_delete_number)?;
                    }
                    SqpkOperation::DeleteData(delete) => {
                        let filename = get_dat_path(
                            target_info.as_ref().unwrap(),
                            delete.main_id,
                            delete.sub_id,
                            delete.file_id,
                        );

                        let new_file =
                            OpenOptions::new().write(true).create(true).open(filename)?;

                        write_empty_file_block_at(
                            &new_file,
                            delete.block_offset,
                            delete.block_number,
                        )?;
                    }
                    SqpkOperation::ExpandData(expand) => {
                        let filename = get_dat_path(
                            target_info.as_ref().unwrap(),
                            expand.main_id,
                            expand.sub_id,
                            expand.file_id,
                        );

                        let (left, _) = filename.rsplit_once('/').unwrap();
                        fs::create_dir_all(left)?;

                        let new_file =
                            OpenOptions::new().write(true).create(true).open(filename)?;

                        write_empty_file_block_at(
                            &new_file,
                            expand.block_offset,
                            expand.block_number,
                        )?;
                    }
                    SqpkOperation::HeaderUpdate(header) => {
                        let file_path = match header.file_kind {
                            TargetFileKind::Dat => get_dat_path(
                                target_info.as_ref().unwrap(),
                                header.main_id,
                                header.sub_id,
                                header.file_id,
                            ),
                            TargetFileKind::Index => get_index_path(
                                target_info.as_ref().unwrap(),
                                header.main_id,
                                header.sub_id,
                                header.file_id,
                            ),
                        };

                        let (left, _) = file_path.rsplit_once('/').ok_or(PatchError::ParseError)?;
                        fs::create_dir_all(left)?;

                        let mut new_file = OpenOptions::new()
                            .write(true)
                            .create(true)
                            .open(file_path)?;

                        if header.header_kind != TargetHeaderKind::Version {
                            new_file.seek(SeekFrom::Start(1024))?;
                        }

                        new_file.write_all(&header.header_data)?;
                    }
                    SqpkOperation::FileOperation(fop) => {
                        let file_path = format!("{}/{}", data_dir, fop.path);
                        let (parent_directory, _) = file_path.rsplit_once('/').unwrap();

                        match fop.operation {
                            SqpkFileOperation::AddFile => {
                                fs::create_dir_all(parent_directory)?;

                                // reverse reading crc32
                                file.seek(SeekFrom::Current(-4))?;

                                let mut data: Vec<u8> = Vec::with_capacity(fop.file_size as usize);

                                while data.len() < fop.file_size as usize {
                                    data.append(&mut read_data_block_patch(&mut file).unwrap());
                                }

                                // re-apply crc32
                                file.seek(SeekFrom::Current(4))?;

                                // now apply the file!
                                let new_file = OpenOptions::new()
                                    .write(true)
                                    .create(true)
                                    .open(&file_path);

                                if let Ok(mut file) = new_file {
                                    if fop.offset == 0 {
                                        file.set_len(0)?;
                                    }

                                    file.seek(SeekFrom::Start(fop.offset))?;
                                    file.write_all(&data)?;
                                } else {
                                    warn!("{file_path} does not exist, skipping.");
                                }
                            }
                            SqpkFileOperation::DeleteFile => {
                                if fs::remove_file(file_path.as_str()).is_err() {
                                    warn!("Failed to remove {file_path}");
                                }
                            }
                            SqpkFileOperation::RemoveAll => {
                                let path: PathBuf =
                                    [data_dir, "sqpack", &get_expansion_folder(fop.expansion_id)]
                                        .iter()
                                        .collect();

                                if fs::read_dir(&path).is_ok() {
                                    fs::remove_dir_all(&path)?;
                                }
                            }
                            SqpkFileOperation::MakeDirTree => {
                                fs::create_dir_all(parent_directory)?;
                            }
                        }
                    }
                    SqpkOperation::PatchInfo(_) => {
                        // Currently, there's nothing we need from PatchInfo. Intentional NOP.
                    }
                    SqpkOperation::TargetInfo(new_target_info) => {
                        target_info = Some(new_target_info);
                    }
                    SqpkOperation::Index(_) => {
                        // Currently, there's nothing we need from Index command. Intentional NOP.
                    }
                }
            }
            ChunkType::FileHeader(_) => {
                // Currently there's nothing very useful in the FileHeader, so it's an intentional NOP.
            }
            ChunkType::ApplyOption(_) => {
                // Currently, IgnoreMissing and IgnoreOldMismatch is not used in XIVQuickLauncher either. This stays as an intentional NOP.
            }
            ChunkType::AddDirectory(_) => {
                debug!("PATCH: NOP AddDirectory");
            }
            ChunkType::DeleteDirectory(_) => {
                debug!("PATCH: NOP DeleteDirectory");
            }
            ChunkType::EndOfFile => {
                return Ok(());
            }
        }
    }
}
