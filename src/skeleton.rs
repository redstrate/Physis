// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use binrw::{binread, until_eof};
use glam::Mat4;
use hard_xml::XmlRead;

use crate::gamedata::MemoryBuffer;

#[binread]
struct SKLB_v1 {
    unk_offset: i16,
    havok_offset: i16
}

#[binread]
struct SKLB_v2 {
    unk_offset: i32,
    havok_offset: i32
}

#[binread]
#[br(magic = 0x736B6C62i32)]
struct SKLB {
    version_one: i16,
    version_two: i16,
    havok_offset: i32,

    #[br(count = havok_offset)]
    raw_header: Vec<u8>,
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
    /// Parses a Havok XML packfile generated by the Havok SDK.
    pub fn from_packfile(buffer: &MemoryBuffer) -> Option<Skeleton> {
        #[derive(XmlRead, Debug)]
        #[xml(tag = "hkpackfile")]
        struct HkPackfile {
            #[xml(child = "hksection")]
            sections: Vec<HkSection>,
            #[xml(attr = "toplevelobject")]
            top_level_object: String,
        }

        #[derive(XmlRead, Debug)]
        #[xml(tag = "hksection")]
        #[allow(dead_code)]
        struct HkSection {
            #[xml(attr = "name")]
            name: String,

            #[xml(child = "hkobject")]
            objects: Vec<HkObject>,
        }

        #[derive(XmlRead, Debug)]
        #[xml(tag = "hkobject")]
        #[allow(dead_code)]
        struct HkObject {
            #[xml(attr = "name")]
            name: Option<String>,

            #[xml(attr = "class")]
            class: Option<String>,

            #[xml(child = "hkparam")]
            params: Vec<HkParam>,
        }

        #[derive(XmlRead, Debug)]
        #[xml(tag = "hkparam")]
        #[allow(dead_code)]
        struct HkParam {
            #[xml(attr = "name")]
            name: String,

            #[xml(attr = "className")]
            class_name: Option<String>,

            #[xml(attr = "variant")]
            variant: Option<String>,

            #[xml(child = "hkobject")]
            objects: Vec<HkObject>,

            #[xml(text)]
            content: String,
        }

        let pak = HkPackfile::from_str(std::str::from_utf8(buffer).unwrap())
            .expect("Failed to parse sidecar file!");

        // find the root level object
        let root_level_object = pak.sections[0]
            .objects
            .iter()
            .find(|s| s.name.as_ref() == Some(&pak.top_level_object))
            .expect("Cannot locate root level object.");

        println!("{:#?}", root_level_object);

        println!("{:#?}", pak);

        None
    }

    /// Parses the TexTools skeleton format, as a nice alternative to packfiles.
    pub fn from_skel(buffer: &MemoryBuffer) -> Option<Skeleton> {
        let mut string_repr = String::from_utf8(buffer.to_vec()).unwrap();

        // for some reason, textools does NOT write valid JSON.
        // let's begin by surrounding all of their json object blocks with an array, which is a valid
        // JSON root.
        string_repr.insert(0, '[');
        string_repr.push(']');

        // then we turn all of newlines into commas, except of course for the last one!
        string_repr = string_repr.replacen('\n', ",", string_repr.matches('\n').count() - 1);

        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        #[serde(rename_all = "PascalCase")]
        #[allow(dead_code)]
        struct BoneObject {
            bone_name: String,
            bone_number: i32,
            bone_parent: i32,
            pose_matrix: [f32; 16],
        }

        let json_bones: Vec<BoneObject> = serde_json::from_str(&string_repr).unwrap();

        let mut skeleton = Skeleton { bones: vec![] };

        for bone in &json_bones {
            let pose_matrix = Mat4::from_cols_array(&bone.pose_matrix);

            let (scale, rotation, translation) = pose_matrix.to_scale_rotation_translation();

            skeleton.bones.push(Bone {
                name: bone.bone_name.clone(),
                parent_index: bone.bone_parent,
                position: translation.to_array(),
                rotation: rotation.to_array(),
                scale: scale.to_array(),
            });
        }

        Some(skeleton)
    }
}
