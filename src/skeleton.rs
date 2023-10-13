// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::upper_case_acronyms)]

use std::io::{Cursor, SeekFrom};
use binrw::{binread, BinRead};
use binrw::helpers::until_eof;
use glam::Mat4;

use crate::gamedata::MemoryBuffer;
use crate::havok::{HavokAnimationContainer, HavokBinaryTagFileReader};

#[binread]
#[br(little)]
struct SklbV1 {
    unk_offset: u16,
    havok_offset: u16,
    body_id: u32,
    mapper_body_id1: u32,
    mapper_body_id2: u32,
    mapper_body_id3: u32,
}

#[binread]
#[br(little)]
struct SklbV2 {
    unk_offset: u32,
    havok_offset: u32,
    unk: u32,
    body_id: u32,
    mapper_body_id1: u32,
    mapper_body_id2: u32,
    mapper_body_id3: u32
}

#[binread]
#[br(magic = 0x736B6C62i32)]
#[br(little)]
struct SKLB {
    version: u32,

    #[br(if(version == 0x3132_3030u32))]
    sklb_v1: Option<SklbV1>,

    #[br(if(version == 0x3133_3030u32 || version == 0x3133_3031u32))]
    sklb_v2: Option<SklbV2>,

    #[br(seek_before(SeekFrom::Start(if (version == 0x3132_3030u32) { sklb_v1.as_ref().unwrap().havok_offset as u64 } else { sklb_v2.as_ref().unwrap().havok_offset as u64 })))]
    #[br(parse_with = until_eof)]
    raw_data: Vec<u8>
}

#[derive(Debug)]
pub struct Bone {
    pub name: String,
    pub parent_index: i32,

    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}

#[derive(Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}

impl Skeleton {
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<Skeleton> {
        let mut cursor = Cursor::new(buffer);

        let sklb = SKLB::read(&mut cursor).unwrap();

        let root = HavokBinaryTagFileReader::read(&sklb.raw_data);
        let raw_animation_container = root.find_object_by_type("hkaAnimationContainer");
        let animation_container = HavokAnimationContainer::new(raw_animation_container);

        let havok_skeleton = &animation_container.skeletons[0];

        let mut skeleton = Skeleton { bones: vec![] };

        for (index, bone) in havok_skeleton.bone_names.iter().enumerate() {
            skeleton.bones.push(Bone {
                name: bone.clone(),
                parent_index: havok_skeleton.parent_indices[index] as i32,
                position: [havok_skeleton.reference_pose[index].translation[0], havok_skeleton.reference_pose[index].translation[1], havok_skeleton.reference_pose[index].translation[2]],
                rotation: havok_skeleton.reference_pose[index].rotation,
                scale: [havok_skeleton.reference_pose[index].scale[0], havok_skeleton.reference_pose[index].scale[1], havok_skeleton.reference_pose[index].scale[2]],
            });
        }

        Some(skeleton)
    }
}
