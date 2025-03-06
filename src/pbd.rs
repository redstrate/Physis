// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, SeekFrom};

use crate::ByteSpan;
use crate::common_file_operations::strings_parser;
use binrw::BinRead;
use binrw::binread;

#[binread]
#[derive(Debug)]
#[br(import { data_offset: i32 })]
#[brw(little)]
#[allow(unused)]
struct RacialDeformer {
    bone_count: i32,

    #[br(count = bone_count)]
    bone_name_offsets: Vec<u16>,

    #[br(args(data_offset as u64, &bone_name_offsets), parse_with = strings_parser)]
    #[br(restore_position)]
    bone_names: Vec<String>,

    #[br(if((bone_count & 1) != 0))]
    #[br(temp)]
    _padding: u16,

    /// 4x3 matrix
    #[br(count = bone_count)]
    #[br(err_context("offset = {} bone count = {}", data_offset, bone_count))]
    transform: Vec<[f32; 12]>,
}

#[binread]
#[derive(Debug)]
#[brw(little)]
struct PreBoneDeformerItem {
    body_id: u16, // the combined body id like 0101
    link_index: i16,
    #[br(pad_after = 4)]
    #[br(temp)]
    data_offset: i32,

    #[br(args { data_offset: data_offset })]
    #[br(seek_before = SeekFrom::Start(data_offset as u64))]
    #[br(restore_position)]
    deformer: RacialDeformer,
}

#[binread]
#[derive(Debug)]
#[brw(little)]
#[allow(dead_code)]
struct PreBoneDeformerLink {
    parent_index: i16,
    first_child_index: i16,
    next_sibling_index: i16,
    deformer_index: u16,
}

#[binread]
#[derive(Debug)]
#[brw(little)]
#[allow(dead_code)]
struct PreBoneDeformerHeader {
    count: i32,

    #[br(count = count)]
    items: Vec<PreBoneDeformerItem>,

    #[br(count = count)]
    links: Vec<PreBoneDeformerLink>,
}

pub struct PreBoneDeformer {
    header: PreBoneDeformerHeader,
}

#[derive(Debug)]
pub struct PreBoneDeformBone {
    /// Name of the affected bone
    pub name: String,
    /// The deform matrix
    pub deform: [f32; 12],
}

#[derive(Debug)]
pub struct PreBoneDeformMatrices {
    /// The prebone deform bones
    pub bones: Vec<PreBoneDeformBone>,
}

impl PreBoneDeformer {
    /// Reads an existing PBD file
    pub fn from_existing(buffer: ByteSpan) -> Option<PreBoneDeformer> {
        let mut cursor = Cursor::new(buffer);
        let header = PreBoneDeformerHeader::read(&mut cursor).ok()?;

        Some(PreBoneDeformer { header })
    }

    /// Calculates the deform matrices between two races
    pub fn get_deform_matrices(
        &self,
        from_body_id: u16,
        to_body_id: u16,
    ) -> Option<PreBoneDeformMatrices> {
        if from_body_id == to_body_id {
            return None;
        }

        let mut item = self
            .header
            .items
            .iter()
            .find(|x| x.body_id == from_body_id)?;
        let mut next = &self.header.links[item.link_index as usize];

        if next.next_sibling_index == -1 {
            return None;
        }

        let mut bones = vec![];

        loop {
            for i in 0..item.deformer.bone_count {
                bones.push(PreBoneDeformBone {
                    name: item.deformer.bone_names[i as usize].clone(),
                    deform: item.deformer.transform[i as usize],
                })
            }

            if next.parent_index == -1 {
                break;
            }

            next = &self.header.links[next.parent_index as usize];
            item = &self.header.items[next.deformer_index as usize];

            if item.body_id == to_body_id {
                break;
            }
        }

        Some(PreBoneDeformMatrices { bones })
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
        PreBoneDeformer::from_existing(&read(d).unwrap());
    }
}
