// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use binrw::{BinRead, BinReaderExt, BinWrite};
use binrw::binrw;
use crate::ByteSpan;

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct PreBoneDeformerItem {
    body_id: u16,
    link_index: u16,
    #[br(pad_after = 4)]
    data_offset: u32
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct PreBoneDeformerLink {
    #[br(pad_after = 4)]
    next_index: i16,
    next_item_index: u16,
}

#[binrw]
#[derive(Debug)]
#[brw(little)]
struct PreBoneDeformerHeader {
    count: u32,

    #[br(count = count)]
    items: Vec<PreBoneDeformerItem>,

    #[br(count = count)]
    links: Vec<PreBoneDeformerLink>,

    #[br(ignore)]
    raw_data: Vec<u8>
}

pub struct PreBoneDeformer {
    header: PreBoneDeformerHeader,
}

#[derive(Debug)]
pub struct PreBoneDeformBone {
    pub name: String,
    pub deform: [f32; 12]
}

#[derive(Debug)]
pub struct PreBoneDeformMatrices {
    pub bones: Vec<PreBoneDeformBone>
}

impl PreBoneDeformer {
    pub fn from_existing(buffer: ByteSpan) -> Option<PreBoneDeformer> {
        let mut cursor = Cursor::new(buffer);
        let mut header = PreBoneDeformerHeader::read(&mut cursor).ok()?;

        header.raw_data = buffer.to_vec();

        Some(PreBoneDeformer {
            header
        })
    }

    pub fn get_deform_matrices(&self, from_body_id: u16, to_body_id: u16) -> Option<PreBoneDeformMatrices> {
        if from_body_id == to_body_id {
            return None;
        }

        let mut item = self.header.items.iter().find(|x| x.body_id == from_body_id)?;
        let mut next = &self.header.links[item.link_index as usize];

        if next.next_index == -1 {
            return None;
        }

        let mut bones = vec![];

        let mut cursor = Cursor::new(&self.header.raw_data);

        loop {
            cursor.seek(SeekFrom::Start(item.data_offset as u64));
            let bone_name_count = cursor.read_le::<u32>().unwrap() as usize;

            let string_offsets_base = item.data_offset as usize + core::mem::size_of::<u32>();

            cursor.seek(SeekFrom::Start(string_offsets_base as u64));
            let mut strings_offset = vec![];
            for i in 0..bone_name_count {
                strings_offset.push(cursor.read_le::<u16>().unwrap());
            }

            let matrices_base = string_offsets_base + (bone_name_count + bone_name_count % 2) * 2;
            cursor.seek(SeekFrom::Start(matrices_base as u64));

            let mut matrices = vec![];
            for i in 0..bone_name_count {
                matrices.push(cursor.read_le::<[f32; 12]>().unwrap());
            }

            for i in 0..bone_name_count {
                let string_offset = item.data_offset as usize + strings_offset[i] as usize;

                let mut string = String::new();

                cursor.seek(SeekFrom::Start(string_offset as u64));
                let mut next_char = cursor.read_le::<u8>().unwrap() as char;
                while next_char != '\0' {
                    string.push(next_char);
                    next_char = cursor.read_le::<u8>().unwrap() as char;
                }

                let matrix = matrices[i];
                bones.push(PreBoneDeformBone {
                    name: string,
                    deform: matrix
                });
            }

            next = &self.header.links[next.next_index as usize];
            item = &self.header.items[next.next_item_index as usize];

            if item.body_id == to_body_id {
                break;
            }
        }

        Some(PreBoneDeformMatrices {
            bones
        })
    }
}
