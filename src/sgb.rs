// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use crate::common::Platform;
use crate::common_file_operations::read_short_identifier;
use crate::common_file_operations::write_short_identifier;
use crate::layer::Layer;
use crate::{ByteSpan, ReadableFile};
use binrw::binrw;
use binrw::{BinRead, BinReaderExt};

#[binrw]
#[derive(Debug)]
struct SgbHeader {
    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    pub identifier: String,

    file_size: i32,
    total_chunk_count: i32,
}

#[binrw]
#[derive(Debug)]
struct SceneChunkHeader {
    #[bw(write_with = write_short_identifier)]
    #[br(parse_with = read_short_identifier)]
    pub identifier: String,

    chunk_size: i32,
    layer_group_offset: i32,
    layer_group_count: i32,
    unknown10: i32,
    unknown14: i32,
    unknown18: i32,
    unknown1c: i32,
    unknown20: i32,
    unknown24: i32,
    unknown28: i32,
    unknown2c: i32,
    unknown30: i32,
    housing_offset: i32,
    unknown38: i32,
    unknown3c: i32,
    unknown40: i32,
    unknown44: i32,
}
/// Shared group binary file, usually with the `.sgb` file extension.
///
/// This is basically a "prefab".
#[derive(Debug)]
pub struct Sgb {
    file_id: String,
    pub chunks: Vec<SgbLayerGroup>,
}

#[binrw]
#[derive(Debug)]
struct SgbLayerGroupHeader {
    layer_group_id: u32,
    name_offset: u32,
    layer_offsets_start: i32,
    layer_offsets_count: i32,
}

#[derive(Debug)]
pub struct SgbLayerGroup {
    pub layer_group_id: u32,
    pub name: String,
    pub layers: Vec<Layer>,
}

impl ReadableFile for Sgb {
    /// Read an existing file.
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let endianness = platform.endianness();
        let mut cursor = Cursor::new(buffer);
        let file_header = SgbHeader::read_options(&mut cursor, endianness, ()).ok()?;

        let start = cursor.position();
        let rewind = start + 8;

        let chunk_header = SceneChunkHeader::read_le(&mut cursor).unwrap();
        if chunk_header.chunk_size <= 0 {
            return Some(Sgb {
                file_id: file_header.identifier,
                chunks: Vec::new(),
            });
        }

        let mut layer_groups: Vec<SgbLayerGroup> =
            Vec::with_capacity(chunk_header.layer_group_count as usize);
        for i in 0..chunk_header.layer_group_count {
            cursor
                .seek(SeekFrom::Start(
                    rewind + (i * 4) as u64 + chunk_header.layer_group_offset as u64,
                ))
                .unwrap();

            let start = cursor.position();
            let group_header = SgbLayerGroupHeader::read_le(&mut cursor).unwrap();
            let end = cursor.position();
            let mut layers: Vec<Layer> =
                Vec::with_capacity(group_header.layer_offsets_count as usize);
            for i in 0..group_header.layer_offsets_count {
                cursor
                    .seek(SeekFrom::Start(
                        start + group_header.layer_offsets_start as u64 + (i as u64 * 4),
                    ))
                    .unwrap();
                let x = cursor.read_le::<i32>().unwrap();
                cursor
                    .seek(SeekFrom::Start(
                        start + group_header.layer_offsets_start as u64 + x as u64,
                    ))
                    .unwrap();
                let layer = Layer::read(endianness, &mut cursor).unwrap();
                layers.push(layer);
            }
            cursor.seek(SeekFrom::Start(end)).unwrap();

            layer_groups.push(SgbLayerGroup {
                layer_group_id: group_header.layer_group_id,
                name: "".to_string(), // TODO fix
                layers: layers,
            })
        }

        Some(Sgb {
            file_id: file_header.identifier,
            chunks: layer_groups,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::LayerEntryData::{LayLight, SharedGroup, BG};
    use crate::layer::LightType::Point;
    use crate::pass_random_invalid;
    use std::fs::read;
    use std::path::PathBuf;

    #[test]
    fn read_simple_sgb() {
        // bg/ffxiv/sea_s1/shared/for_bg/sgbg_s1t0_a0_lmp1.sgb
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("sgbg_s1t0_a0_lmp1.sgb");

        let sgb = Sgb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(sgb.file_id, "SGB1");
        assert_eq!(sgb.chunks.len(), 1);

        let chunk = &sgb.chunks[0];
        assert_eq!(chunk.layers.len(), 1);

        let layer = &chunk.layers[0];
        assert_eq!(layer.header.layer_id, 65537);
        assert_eq!(layer.header.name.value, "sgbg_s1t0_a0_lmp1");
        assert_eq!(layer.header.festival_id, 0);
        assert_eq!(layer.header.instance_object_count, 2);

        let bg_object = &layer.objects[0];
        matches!(&bg_object.data, BG(data) if data.asset_path.value == "bg/ffxiv/sea_s1/twn/common/bgparts/s1t0_a0_lmp1.mdl");

        let light_object = &layer.objects[1];
        matches!(&light_object.data, LayLight(data) if data.light_type == Point && data.texture_path_offset == 425);
    }

    #[test]
    fn load_npc_sgb() {
        // bgcommon/world/mpc/shared/for_bg/sgbg_w_mpc_002_11a.sgb
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("sgbg_w_mpc_002_11a.sgb");

        let sgb = Sgb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(sgb.file_id, "SGB1");
        assert_eq!(sgb.chunks.len(), 1);

        let chunk = &sgb.chunks[0];
        assert_eq!(chunk.layers.len(), 1);

        let layer = &chunk.layers[0];
        assert_eq!(layer.header.layer_id, 65537);
        assert_eq!(layer.header.name.value, "sgbg_w_mpc_002_11a");
        assert_eq!(layer.header.festival_id, 0);
        assert_eq!(layer.header.instance_object_count, 1);

        let nested_sgb_object = &layer.objects[0];
        matches!(&nested_sgb_object.data, SharedGroup(data) if data.asset_path.value == "bgcommon/world/mpc/shared/for_bg/sgbg_w_mpc_002_01a.sgb");
    }

    #[test]
    fn load_aet_sgb() {
        // bgcommon/world/mpc/shared/for_bg/sgbg_w_aet_001_01a.sgb
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("sgbg_w_aet_001_01a.sgb");

        let sgb = Sgb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(sgb.file_id, "SGB1");
        assert_eq!(sgb.chunks.len(), 1);

        let chunk = &sgb.chunks[0];
        assert_eq!(chunk.layers.len(), 1);

        let layer = &chunk.layers[0];
        assert_eq!(layer.header.layer_id, 65537);
        assert_eq!(layer.header.name.value, "sgbg_w_aet_001_01a");
        assert_eq!(layer.header.festival_id, 0);
        assert_eq!(layer.header.instance_object_count, 18);

        let bg_object = &layer.objects[1];
        matches!(&bg_object.data, BG(data) if data.asset_path.value == "bgcommon/world/aet/001/bgparts/w_aet_001_01a.mdl");
    }

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Sgb>();
    }
}
