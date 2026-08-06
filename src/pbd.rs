// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, SeekFrom};

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use crate::common_file_operations::strings_parser;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

#[binrw]
#[derive(Debug)]
#[br(import { data_offset: i32 })]
#[allow(unused)]
pub struct RacialDeformer {
    bone_count: u32,

    #[br(count = bone_count)]
    bone_name_offsets: Vec<u16>,

    #[br(args(data_offset as u64, &bone_name_offsets), parse_with = strings_parser)]
    #[br(restore_position)]
    #[bw(ignore)]
    bone_names: Vec<String>,

    #[br(if((bone_count & 1) != 0))]
    #[br(temp)]
    #[bw(ignore)]
    _padding: u16,

    /// A 4x3 transformation matrix.
    #[br(count = bone_count)]
    pub transform: Vec<[f32; 12]>,
}

#[binrw]
#[derive(Debug)]
pub struct PreBoneDeformerItem {
    /// The combined body id like `0101`.
    pub body_id: u16,
    pub link_index: u16,
    data_offset: i32,
    unk_scale: f32,

    /// Some bodies like 101 don't have a deformer.
    #[br(if(data_offset > 0), seek_before = SeekFrom::Start(data_offset as u64), args { data_offset: data_offset }, restore_position)]
    pub deformer: Option<RacialDeformer>,
}

#[binrw]
#[derive(Debug)]
#[allow(dead_code)]
pub struct PreBoneDeformerLink {
    pub parent_index: i16,
    pub first_child_index: i16,
    pub next_sibling_index: i16,
    pub deformer_index: u16,
}

/// Pre-bone deformer file, usually with the `.pbd` file extension.
///
/// Used to transform or "deform" a base skeleton. For example, various races use pre-bone deformers to create their unique body shapes.
#[binrw]
#[derive(Debug)]
pub struct PreBoneDeformer {
    #[br(temp)]
    #[bw(calc = (items.len() + links.len()) as u32)]
    entry_count: u32,

    #[br(count = entry_count)]
    pub items: Vec<PreBoneDeformerItem>,

    #[br(count = entry_count)]
    pub links: Vec<PreBoneDeformerLink>,
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

impl ReadableFile for PreBoneDeformer {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(PreBoneDeformer::read_options(
            &mut cursor,
            platform.endianness(),
            (),
        )?)
    }
}

impl WritableFile for PreBoneDeformer {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

impl PreBoneDeformer {
    /// Calculates the deform matrices between two races
    pub fn get_deform_matrices(
        &self,
        from_body_id: u16,
        to_body_id: u16,
    ) -> Option<PreBoneDeformMatrices> {
        if from_body_id == to_body_id {
            return None;
        }

        let mut item = self.items.iter().find(|x| x.body_id == from_body_id)?;
        let mut next = &self.links[item.link_index as usize];

        if next.next_sibling_index == -1 {
            return None;
        }

        let Some(deformer) = &item.deformer else {
            return None;
        };

        let mut bones = vec![];

        loop {
            for i in 0..deformer.bone_count {
                bones.push(PreBoneDeformBone {
                    name: deformer.bone_names[i as usize].clone(),
                    deform: deformer.transform[i as usize],
                })
            }

            if next.parent_index == -1 {
                break;
            }

            next = &self.links[next.parent_index as usize];
            item = &self.items[next.deformer_index as usize];

            if item.body_id == to_body_id {
                break;
            }
        }

        Some(PreBoneDeformMatrices { bones })
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<PreBoneDeformer>();
    }
}
