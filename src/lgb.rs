// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use binrw::{BinRead, BinReaderExt, BinWrite, binrw};

use crate::{
    ByteBuffer, ByteSpan, ReadableFile, WritableFile,
    common::Platform,
    layer::Layer,
    string_heap::{HeapString, StringHeap},
};

/// "LGB1"
pub const LGB1_ID: u32 = u32::from_le_bytes(*b"LGB1");
/// "LGP1"
pub const LGP1_ID: u32 = u32::from_le_bytes(*b"LGP1");

#[binrw]
#[derive(Debug)]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LgbHeader {
    // Example: "LGB1"
    file_id: u32,
    // File size *including* this header
    file_size: i32,
    total_chunk_count: i32,
}

#[binrw]
#[derive(Debug)]
#[br(import(string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap))]
#[allow(dead_code)] // most of the fields are unused at the moment
struct LayerChunkHeader {
    chunk_id: u32,
    chunk_size: i32,
    layer_group_id: i32,
    #[brw(args(string_heap))]
    pub name: HeapString,
    layer_offset: i32,
    layer_count: i32,
}

impl LayerChunkHeader {
    const SIZE: usize = 24;
}

#[derive(Debug)]
pub struct LayerChunk {
    // Example: "LGP1"
    pub chunk_id: u32,
    pub layer_group_id: i32,
    pub name: String,
    pub layers: Vec<Layer>,
}

/// Layer group binary file, usually with the `.lgb` file extension.
///
/// Contains information about where game objects are placed.
#[derive(Debug)]
pub struct Lgb {
    pub file_id: u32,
    pub chunks: Vec<LayerChunk>,
}

impl ReadableFile for Lgb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> Option<Self> {
        let mut cursor = Cursor::new(buffer);
        let endianness = platform.endianness();

        let file_header = LgbHeader::read_options(&mut cursor, endianness, ()).unwrap();
        if file_header.file_size <= 0 || file_header.total_chunk_count <= 0 {
            return None;
        }

        // yes, for some reason it begins at 8 bytes in?!?!
        let chunk_string_heap = StringHeap::from(cursor.position() + 8);

        let chunk_header =
            LayerChunkHeader::read_options(&mut cursor, endianness, (&chunk_string_heap,)).unwrap();
        if chunk_header.chunk_size <= 0 {
            return Some(Lgb {
                file_id: file_header.file_id,
                chunks: Vec::new(),
            });
        }

        let old_pos = cursor.position();

        let mut layer_offsets = vec![0i32; chunk_header.layer_count as usize];
        for i in 0..chunk_header.layer_count {
            layer_offsets[i as usize] = cursor.read_type_args::<i32>(endianness, ()).unwrap();
        }

        let mut layers = Vec::new();

        for i in 0..chunk_header.layer_count {
            cursor
                .seek(SeekFrom::Start(old_pos + layer_offsets[i as usize] as u64))
                .unwrap();

            let layer = Layer::read(endianness, &mut cursor)?;
            layers.push(layer);
        }

        let layer_chunk = LayerChunk {
            chunk_id: chunk_header.chunk_id,
            layer_group_id: chunk_header.layer_group_id,
            name: chunk_header.name.value,
            layers,
        };

        Some(Lgb {
            file_id: file_header.file_id,
            chunks: vec![layer_chunk],
        })
    }
}

impl WritableFile for Lgb {
    fn write_to_buffer(&self, _platform: Platform) -> Option<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);

            // skip header, will be writing it later
            cursor
                .seek(SeekFrom::Start(std::mem::size_of::<LgbHeader>() as u64))
                .unwrap();

            // base offset for deferred data
            let mut data_base = cursor.stream_position().unwrap();

            let mut chunk_data_heap = StringHeap {
                pos: data_base + 4,
                bytes: Vec::new(),
                free_pos: data_base + 4,
            };

            let mut chunk_string_heap = StringHeap {
                pos: data_base + 4,
                bytes: Vec::new(),
                free_pos: data_base + 4,
            };

            // we will write this later, when we have a working string heap
            let layer_chunk_header_pos = cursor.stream_position().unwrap();
            cursor
                .seek(SeekFrom::Current(LayerChunkHeader::SIZE as i64))
                .unwrap();

            // skip offsets for now, they will be written later
            let offset_pos = cursor.position();
            cursor
                .seek(SeekFrom::Current(
                    (std::mem::size_of::<i32>() * self.chunks[0].layers.len()) as i64,
                ))
                .ok()?;

            let mut offsets: Vec<i32> = Vec::new();

            let layer_data_offset = cursor.position();

            // first pass: write layers, we want to get a correct *chunk_data_heap*
            for layer in &self.chunks[0].layers {
                // set offset
                // this is also used to reference positions inside this layer
                let layer_offset = cursor.position() as i32;
                offsets.push(layer_offset);

                layer
                    .header
                    .write_le_args(&mut cursor, (&mut chunk_data_heap, &mut chunk_string_heap))
                    .ok()?;

                for obj in &layer.objects {
                    obj.write_le_args(&mut cursor, (&mut chunk_string_heap,))
                        .ok()?;
                }
            }

            // make sure the heaps are at the end of the layer data
            data_base += cursor.stream_position().unwrap() - layer_data_offset
                + (offsets.len() * std::mem::size_of::<u32>()) as u64;

            // second pass: write layers again, we want to get a correct *chunk_string_heap* now that we know of the size of chunk_data_heap
            chunk_string_heap = StringHeap {
                pos: data_base + 4 + chunk_data_heap.bytes.len() as u64,
                bytes: Vec::new(),
                free_pos: data_base + 4 + chunk_data_heap.bytes.len() as u64,
            };
            chunk_data_heap = StringHeap {
                pos: data_base + 4,
                bytes: Vec::new(),
                free_pos: data_base + 4,
            };

            // write header now, because it has a string
            cursor
                .seek(SeekFrom::Start(layer_chunk_header_pos))
                .unwrap();

            // TODO: support multiple layer chunks
            let layer_chunk = LayerChunkHeader {
                chunk_id: self.chunks[0].chunk_id,
                chunk_size: 24, // double lol
                layer_group_id: self.chunks[0].layer_group_id,
                name: HeapString {
                    value: self.chunks[0].name.clone(),
                },
                layer_offset: 16, // lol
                layer_count: self.chunks[0].layers.len() as i32,
            };
            layer_chunk
                .write_le_args(&mut cursor, (&mut chunk_string_heap,))
                .ok()?;

            // now write the layer data for the final time
            cursor.seek(SeekFrom::Start(layer_data_offset)).unwrap();
            for layer in &self.chunks[0].layers {
                chunk_data_heap.free_pos = layer_data_offset + 12; // 52
                chunk_string_heap.free_pos =
                    chunk_data_heap.free_pos + chunk_string_heap.bytes.len() as u64 + 16;

                layer
                    .header
                    .write_le_args(&mut cursor, (&mut chunk_data_heap, &mut chunk_string_heap))
                    .ok()?;

                for obj in &layer.objects {
                    obj.write_le_args(&mut cursor, (&mut chunk_string_heap,))
                        .ok()?;
                }
            }

            // write the heaps
            chunk_data_heap.write_le(&mut cursor).ok()?;
            chunk_string_heap.write_le(&mut cursor).ok()?;

            // write offsets
            assert_eq!(offsets.len(), self.chunks[0].layers.len());
            cursor.seek(SeekFrom::Start(offset_pos)).ok()?;
            for offset in offsets {
                // TODO: im probably subtracting from the wrong offset
                (offset - offset_pos as i32).write_le(&mut cursor).ok()?;
            }
        }

        let file_size = buffer.len() as i32;

        {
            let mut cursor = Cursor::new(&mut buffer);

            // write the header, now that we now the file size
            cursor.seek(SeekFrom::Start(0)).ok()?;
            let lgb_header = LgbHeader {
                file_id: self.file_id,
                file_size,
                total_chunk_count: self.chunks.len() as i32,
            };
            lgb_header.write_le(&mut cursor).ok()?;
        }

        Some(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use crate::layer::{
        LayerHeader, LayerSetReferenced, LayerSetReferencedList, LayerSetReferencedType,
    };
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Lgb>();
    }

    #[test]
    fn read_empty_planlive() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty_planlive.lgb");

        let lgb = Lgb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(lgb.file_id, LGB1_ID);
        assert_eq!(lgb.chunks.len(), 1);

        let chunk = &lgb.chunks[0];
        assert_eq!(chunk.chunk_id, LGP1_ID);
        assert_eq!(chunk.layer_group_id, 261);
        assert_eq!(chunk.name, "PlanLive".to_string());
        assert!(chunk.layers.is_empty());
    }

    #[test]
    fn write_empty_planlive() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty_planlive.lgb");

        let good_lgb_bytes = read(d).unwrap();

        let lgb = Lgb {
            file_id: LGB1_ID,
            chunks: vec![LayerChunk {
                chunk_id: LGP1_ID,
                layer_group_id: 261,
                name: "PlanLive".to_string(),
                layers: Vec::new(),
            }],
        };
        assert_eq!(
            lgb.write_to_buffer(Platform::Win32).unwrap(),
            good_lgb_bytes
        );
    }

    #[test]
    fn read_simple_planevent() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("simple_planevent.lgb");

        let lgb = Lgb::from_existing(Platform::Win32, &read(d).unwrap()).unwrap();
        assert_eq!(lgb.file_id, LGB1_ID);
        assert_eq!(lgb.chunks.len(), 1);

        let chunk = &lgb.chunks[0];
        assert_eq!(chunk.chunk_id, LGP1_ID);
        assert_eq!(chunk.layer_group_id, 260);
        assert_eq!(chunk.name, "PlanEvent".to_string());
        assert_eq!(
            chunk.layers,
            vec![Layer {
                header: LayerHeader {
                    layer_id: 133894,
                    name: HeapString {
                        value: "QST_StmBdr102".to_string()
                    },
                    instance_object_offset: 52,
                    instance_object_count: 0,
                    tool_mode_visible: true,
                    tool_mode_read_only: false,
                    is_bush_layer: false,
                    ps3_visible: true,
                    layer_set_referenced_list: LayerSetReferencedList {
                        referenced_type: LayerSetReferencedType::Include,
                        layer_sets: vec![LayerSetReferenced {
                            layer_set_id: 132261
                        }]
                    },
                    festival_id: 0,
                    festival_phase_id: 0,
                    is_temporary: 0,
                    is_housing: 0,
                    version_mask: 47,
                    ob_set_referenced_list: 68,
                    ob_set_referenced_list_count: 0,
                    ob_set_enable_referenced_list: 68,
                    ob_set_enable_referenced_list_count: 0
                },
                objects: vec![]
            }]
        );
    }

    #[test]
    fn write_simple_planevent() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("simple_planevent.lgb");

        let good_lgb_bytes = read(d).unwrap();

        let lgb = Lgb {
            file_id: LGB1_ID,
            chunks: vec![LayerChunk {
                chunk_id: LGP1_ID,
                layer_group_id: 260,
                name: "PlanEvent".to_string(),
                layers: vec![Layer {
                    header: LayerHeader {
                        layer_id: 133894,
                        name: HeapString {
                            value: "QST_StmBdr102".to_string(),
                        },
                        instance_object_offset: 52,
                        instance_object_count: 0,
                        tool_mode_visible: true,
                        tool_mode_read_only: false,
                        is_bush_layer: false,
                        ps3_visible: true,
                        layer_set_referenced_list: LayerSetReferencedList {
                            referenced_type: LayerSetReferencedType::Include,
                            layer_sets: vec![LayerSetReferenced {
                                layer_set_id: 132261,
                            }],
                        },
                        festival_id: 0,
                        festival_phase_id: 0,
                        is_temporary: 0,
                        is_housing: 0,
                        version_mask: 47,
                        ob_set_referenced_list: 68,
                        ob_set_referenced_list_count: 0,
                        ob_set_enable_referenced_list: 68,
                        ob_set_enable_referenced_list_count: 0,
                    },
                    objects: vec![],
                }],
            }],
        };
        assert_eq!(
            lgb.write_to_buffer(Platform::Win32).unwrap(),
            good_lgb_bytes
        );
    }
}
