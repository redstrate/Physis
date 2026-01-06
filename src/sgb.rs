// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use crate::common::Platform;
use crate::common_file_operations::read_short_identifier;
use crate::common_file_operations::write_short_identifier;
use crate::layer::Layer;
use crate::layer::ScnSection;
use crate::string_heap::StringHeap;
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

/// Shared group binary file, usually with the `.sgb` file extension.
///
/// This is basically a "prefab".
#[derive(Debug)]
pub struct Sgb {
    pub file_id: String,
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

        let string_heap = StringHeap::from(0); // TODO: wrong position probably

        let chunk_header =
            ScnSection::read_options(&mut cursor, endianness, (&string_heap,)).unwrap();

        let mut layer_groups: Vec<SgbLayerGroup> =
            Vec::with_capacity(chunk_header.num_layer_groups as usize);
        for i in 0..chunk_header.num_layer_groups {
            cursor
                .seek(SeekFrom::Start(
                    rewind + (i * 4) as u64 + chunk_header.offset_layer_groups as u64,
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
                layers,
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
    use crate::pass_random_invalid;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Sgb>();
    }
}
