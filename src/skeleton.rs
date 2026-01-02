// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

#![allow(unused)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::upper_case_acronyms)]

use binrw::helpers::until_eof;
use binrw::{BinRead, binread};
use std::io::{Cursor, SeekFrom};

use crate::common::Platform;
use crate::havok::{HavokAnimationContainer, HavokBinaryTagFileReader};
use crate::{ByteSpan, ReadableFile};

#[binread]
struct SklbV1 {
    unk_offset: u16,
    havok_offset: u16,
    body_id: u32,
    mapper_body_id1: u32,
    mapper_body_id2: u32,
    mapper_body_id3: u32,
}

#[binread]
struct SklbV2 {
    unk_offset: u32,
    havok_offset: u32,
    unk: u32,
    body_id: u32,
    mapper_body_id1: u32,
    mapper_body_id2: u32,
    mapper_body_id3: u32,
}

#[binread]
#[br(magic = 0x736B6C62i32)]
struct SKLB {
    version: u32,

    #[br(if(version == 0x3132_3030u32))]
    sklb_v1: Option<SklbV1>,

    #[br(if(version == 0x3133_3030u32 || version == 0x3133_3031u32))]
    sklb_v2: Option<SklbV2>,

    #[br(seek_before(SeekFrom::Start(if (version == 0x3132_3030u32) { sklb_v1.as_ref().unwrap().havok_offset as u64 } else { sklb_v2.as_ref().unwrap().havok_offset as u64 })))]
    #[br(parse_with = until_eof)]
    raw_data: Vec<u8>,
}

#[derive(Debug)]
pub struct Bone {
    /// Name of the bone
    pub name: String,
    /// Index of the parent bone in the Skeleton's `bones` vector
    pub parent_index: i32,

    /// Position of the bone
    pub position: [f32; 3],
    /// Rotation quanternion of the bone
    pub rotation: [f32; 4],
    /// Scale of the bone
    pub scale: [f32; 3],
}

/// Skeleton file, usually with the `.sklb` file extension.
///
/// Contains a tree of bones.
#[derive(Debug)]
pub struct Skeleton {
    /// Bones of this skeleton
    pub bones: Vec<Bone>,
}

impl ReadableFile for Skeleton {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Skeleton> {
        let mut cursor = Cursor::new(buffer);

        let sklb = SKLB::read_options(&mut cursor, platform.endianness(), ()).ok()?;

        let root = HavokBinaryTagFileReader::read(&sklb.raw_data);
        let raw_animation_container = root.find_object_by_type("hkaAnimationContainer");
        let animation_container = HavokAnimationContainer::new(raw_animation_container);

        let havok_skeleton = &animation_container.skeletons[0];

        let mut skeleton = Skeleton { bones: vec![] };

        for (index, bone) in havok_skeleton.bone_names.iter().enumerate() {
            skeleton.bones.push(Bone {
                name: bone.clone(),
                parent_index: havok_skeleton.parent_indices[index] as i32,
                position: [
                    havok_skeleton.reference_pose[index].translation[0],
                    havok_skeleton.reference_pose[index].translation[1],
                    havok_skeleton.reference_pose[index].translation[2],
                ],
                rotation: havok_skeleton.reference_pose[index].rotation,
                scale: [
                    havok_skeleton.reference_pose[index].scale[0],
                    havok_skeleton.reference_pose[index].scale[1],
                    havok_skeleton.reference_pose[index].scale[2],
                ],
            });
        }

        Some(skeleton)
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
        Skeleton::from_existing(Platform::Win32, &read(d).unwrap());
    }
}
