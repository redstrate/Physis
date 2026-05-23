// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::{Cursor, Seek, SeekFrom};

use binrw::{BinRead, BinReaderExt, BinWrite, binrw};

use crate::{
    ByteBuffer, ByteSpan, ReadableFile, WritableFile,
    common::Platform,
    layer::Layer,
    string_heap::{HeapPointer, HeapString, StringHeap},
};

#[binrw]
#[derive(Debug, Default)]
#[allow(dead_code)] // most of the fields are unused at the moment
#[brw(magic = b"LGB1")]
struct LgbHeader {
    // File size *including* this header
    file_size: i32,
    total_chunk_count: i32,
}

impl LgbHeader {
    pub(crate) const SIZE: usize = 0x0C;
}

#[binrw]
#[derive(Debug, Default)]
#[br(import(string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap), stream = w)]
#[allow(dead_code)] // most of the fields are unused at the moment
#[brw(magic = b"LGP1")]
struct LayerChunkHeader {
    chunk_size: i32,

    #[br(temp)]
    #[bw(calc = HeapPointer::from_stream(w))]
    heap_pointer: HeapPointer,

    layer_group_id: i32,

    /// Name of this layer.
    #[brw(args(heap_pointer, string_heap))]
    pub name: HeapString,

    layer_offset: i32,
    layer_count: i32,
}

impl LayerChunkHeader {
    const SIZE: usize = 24;
}

#[derive(Debug)]
pub struct LayerChunk {
    /// The ID of this chunk.
    pub layer_group_id: i32,
    /// Name of this layer chunk.
    pub name: String,
    /// The layers in this chunk.
    pub layers: Vec<Layer>,
}

/// Layer group binary file, usually with the `.lgb` file extension.
///
/// Contains information about where game objects are placed.
#[derive(Debug)]
pub struct Lgb {
    pub chunks: Vec<LayerChunk>,
}

impl ReadableFile for Lgb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        let endianness = platform.endianness();

        let string_heap = StringHeap::from(cursor.position() as i64);
        let data_heap = StringHeap::from(cursor.position() as i64);

        let file_header = LgbHeader::read_options(&mut cursor, endianness, ())?;
        if file_header.file_size <= 0 || file_header.total_chunk_count <= 0 {
            return Err(crate::Error::InvalidFile);
        }

        // This actually matches client behavior, because of course they have padding here.
        loop {
            let magic = <[u8; 4]>::read_options(&mut cursor, endianness, ())?;
            if &magic == b"LGP1" {
                cursor.seek(SeekFrom::Current(-4))?;
                break;
            }
        }

        let chunk_header =
            LayerChunkHeader::read_options(&mut cursor, endianness, (&string_heap,))?;
        if chunk_header.chunk_size <= 0 {
            return Ok(Lgb { chunks: Vec::new() });
        }

        let old_pos = cursor.position();

        let mut layer_offsets = vec![0i32; chunk_header.layer_count as usize];
        for i in 0..chunk_header.layer_count {
            layer_offsets[i as usize] = cursor.read_type_args::<i32>(endianness, ())?;
        }

        let mut layers = Vec::new();

        for i in 0..chunk_header.layer_count {
            cursor.seek(SeekFrom::Start(old_pos + layer_offsets[i as usize] as u64))?;

            let layer = Layer::read(endianness, &mut cursor, &data_heap, &string_heap)?;
            layers.push(layer);
        }

        let layer_chunk = LayerChunk {
            layer_group_id: chunk_header.layer_group_id,
            name: chunk_header.name.value,
            layers,
        };

        Ok(Lgb {
            chunks: vec![layer_chunk],
        })
    }
}

impl WritableFile for Lgb {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();
        let endian = platform.endianness();

        {
            let mut cursor = Cursor::new(&mut buffer);

            // skip header, will be writing it later
            cursor.seek(SeekFrom::Start(LgbHeader::SIZE as u64))?;

            // we don't store the positions for this pass, so the defaults are fine.
            let mut chunk_data_heap = StringHeap::default();
            let mut chunk_string_heap = StringHeap::default();

            // we will write this later, when we have a working string heap
            let layer_chunk_header_pos = cursor.stream_position()?;
            cursor.seek(SeekFrom::Current(LayerChunkHeader::SIZE as i64))?;

            // skip offsets for now, they will be written later
            let offset_pos = cursor.position();
            cursor.seek(SeekFrom::Current(
                (std::mem::size_of::<i32>() * self.chunks[0].layers.len()) as i64,
            ))?;

            let mut layer_offsets: Vec<i32> = Vec::new();
            let mut object_offsets: Vec<i32> = Vec::new();

            let layer_data_offset = cursor.position();

            // first pass: write layers, we want to get a correct *chunk_data_heap*
            for layer in &self.chunks[0].layers {
                // set offset
                // this is also used to reference positions inside this layer
                layer_offsets.push(cursor.position() as i32);

                layer.header.write_options(
                    &mut cursor,
                    endian,
                    (&mut chunk_data_heap, &mut chunk_string_heap),
                )?;

                let object_offset_base = cursor.stream_position()? as i32;

                // Skip offsets that will be written later.
                cursor.seek(SeekFrom::Current(
                    layer.objects.len() as i64 * std::mem::size_of::<u32>() as i64,
                ))?;

                for obj in &layer.objects {
                    object_offsets.push(cursor.stream_position()? as i32 - object_offset_base);

                    obj.write_options(&mut cursor, endian, (&mut chunk_string_heap,))?;
                }
            }

            // make sure the heaps are at the end of the layer data:
            // TODO: this logic doesn't make much sense...
            let data_offset = cursor.stream_position()?
                + (if !layer_offsets.is_empty() {
                    LgbHeader::SIZE as u64
                } else {
                    0
                });

            // second pass: write layers again, we want to get a correct *chunk_string_heap* now that we know of the size of chunk_data_heap
            chunk_data_heap = StringHeap {
                free_pos: data_offset,
                ..Default::default()
            };
            chunk_string_heap = StringHeap {
                free_pos: data_offset + chunk_data_heap.bytes.len() as u64,
                ..Default::default()
            };

            // write header now, because it has a string
            cursor.seek(SeekFrom::Start(layer_chunk_header_pos))?;

            // TODO: support multiple layer chunks
            let layer_chunk = LayerChunkHeader {
                chunk_size: 24, // double lol
                layer_group_id: self.chunks[0].layer_group_id,
                name: HeapString {
                    value: self.chunks[0].name.clone(),
                },
                layer_offset: 16, // lol
                layer_count: self.chunks[0].layers.len() as i32,
            };
            layer_chunk.write_options(&mut cursor, endian, (&mut chunk_string_heap,))?;

            // now write the layer data for the final time
            cursor.seek(SeekFrom::Start(layer_data_offset))?;
            for layer in &self.chunks[0].layers {
                // Write the correct amount of objects and their offsets now
                let mut new_header = layer.header.clone();
                new_header.instance_object_count = layer.objects.len() as i32;
                new_header.instance_object_offset = 52; // TODO: placeholder

                new_header.write_options(
                    &mut cursor,
                    endian,
                    (&mut chunk_data_heap, &mut chunk_string_heap),
                )?;

                object_offsets.write_options(&mut cursor, endian, ())?;

                for obj in &layer.objects {
                    obj.write_options(&mut cursor, endian, (&mut chunk_string_heap,))?;
                }
            }

            // write the heaps
            chunk_data_heap.write_options(&mut cursor, endian, ())?;
            chunk_string_heap.write_options(&mut cursor, endian, ())?;

            // write offsets
            assert_eq!(layer_offsets.len(), self.chunks[0].layers.len());
            cursor.seek(SeekFrom::Start(offset_pos))?;
            for offset in layer_offsets {
                // TODO: im probably subtracting from the wrong offset
                (offset - offset_pos as i32).write_options(&mut cursor, endian, ())?;
            }
        }

        let file_size = buffer.len() as i32;

        {
            let mut cursor = Cursor::new(&mut buffer);

            // write the header, now that we now the file size
            cursor.seek(SeekFrom::Start(0))?;
            let lgb_header = LgbHeader {
                file_size,
                total_chunk_count: self.chunks.len() as i32,
            };
            lgb_header.write_options(&mut cursor, endian, ())?;
        }

        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::read;
    use std::path::PathBuf;

    use crate::common::ensure_size;
    use crate::layer::{
        InstanceObject, LayerEntryData, LayerHeader, LayerSetReferenced, LayerSetReferencedList,
        LayerSetReferencedType, SoundEffectType, SoundInstanceObject, SoundParameters,
        Transformation,
    };
    use crate::pass_random_invalid;
    use crate::string_heap::HeapString;

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
        assert_eq!(lgb.chunks.len(), 1);

        let chunk = &lgb.chunks[0];
        assert_eq!(chunk.layer_group_id, 261);
        assert_eq!(chunk.name, "PlanLive".to_string());
        assert!(chunk.layers.is_empty());
    }

    #[test]
    fn read_padded_planlive() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("padded_planner.lgb");

        // NOTE: It would be nice to read these eventually, but I'm pretty sure all cases are empty and useless.
        // So the best we could do right now is not panic while reading them.
        assert!(Lgb::from_existing(Platform::Win32, &read(d).unwrap()).is_err());
    }

    #[test]
    fn write_empty_planlive() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty_planlive.lgb");

        let good_lgb_bytes = read(d).unwrap();

        let lgb = Lgb {
            chunks: vec![LayerChunk {
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
        assert_eq!(lgb.chunks.len(), 1);

        let chunk = &lgb.chunks[0];
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
                    visible: true,
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
                    is_temporary: false,
                    is_housing: false,
                    version_mask: 47,
                    object_set_referenced: vec![],
                    object_set_enable_referenced: vec![],
                },
                objects: vec![]
            }]
        );
    }

    #[ignore = "We most likely do not write the data heap correctly right now."]
    #[test]
    fn write_simple_planevent() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("simple_planevent.lgb");

        let good_lgb_bytes = read(d).unwrap();

        let lgb = Lgb {
            chunks: vec![LayerChunk {
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
                        visible: true,
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
                        is_temporary: false,
                        is_housing: false,
                        version_mask: 47,
                        object_set_referenced: vec![],
                        object_set_enable_referenced: vec![],
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

    #[test]
    fn test_lgbheader_size() {
        ensure_size::<LgbHeader, { LgbHeader::SIZE }>();
    }

    #[test]
    fn test_layerchunkheader_size() {
        // FIXME: Needs StringHeap
        // ensure_size::<LayerChunkHeader, { LayerChunkHeader::SIZE }>();
    }

    #[test]
    fn write_simple_sound() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("simple_sound.lgb");

        let good_lgb_bytes = read(d).unwrap();

        let lgb = Lgb {
            chunks: vec![LayerChunk {
                layer_group_id: 262,
                name: "Sound".to_string(),
                layers: vec![Layer {
                    header: LayerHeader {
                        layer_id: 253922,
                        name: HeapString {
                            value: "Spot".to_string(),
                        },
                        instance_object_offset: 52,
                        instance_object_count: 1,
                        visible: true,
                        tool_mode_read_only: false,
                        is_bush_layer: false,
                        ps3_visible: true,
                        layer_set_referenced_list: LayerSetReferencedList {
                            referenced_type: LayerSetReferencedType::All,
                            layer_sets: vec![],
                        },
                        festival_id: 0,
                        festival_phase_id: 0,
                        is_temporary: false,
                        is_housing: false,
                        version_mask: 0,
                        object_set_referenced: vec![],
                        object_set_enable_referenced: vec![],
                    },
                    objects: vec![InstanceObject {
                        instance_id: 8338105,
                        name: HeapString::default(),
                        transform: Transformation {
                            translation: [83.76575, 68.86192, 158.3914],
                            rotation: [0.0, 0.0, 0.0],
                            scale: [15.0, 4.989685, 60.0],
                        },
                        data: LayerEntryData::Sound(SoundInstanceObject {
                            asset_path: HeapString {
                                value: "bgcommon/sound/spot_wnd/spot_wnd_Blizard_Loop_s.scd"
                                    .to_string(),
                            },
                            parameters: SoundParameters {
                                sound_effect_type: SoundEffectType::Surface,
                                auto_play: true,
                                is_no_far_clip: false,
                                unk1: 0,
                                binary_offset: 20,
                                binary_count: 160,
                                point_selection: 0,
                                binaries: vec![
                                    160, 0, 7, 1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 128,
                                    63, 0, 0, 128, 63, 0, 0, 128, 63, 0, 0, 128, 63, 222, 81, 147,
                                    194, 11, 191, 137, 66, 44, 221, 130, 67, 0, 0, 128, 63, 171,
                                    126, 61, 67, 150, 184, 137, 66, 201, 222, 130, 67, 0, 0, 128,
                                    63, 91, 129, 61, 67, 60, 179, 137, 66, 171, 143, 4, 194, 0, 0,
                                    128, 63, 131, 76, 147, 194, 179, 185, 137, 66, 148, 156, 4,
                                    194, 0, 0, 128, 63, 0, 0, 112, 65, 0, 0, 112, 66, 128, 171,
                                    159, 64, 128, 171, 159, 64, 0, 0, 128, 63, 184, 30, 69, 63, 0,
                                    0, 128, 63, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0,
                                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                ],
                            },
                        }),
                    }],
                }],
            }],
        };
        assert_eq!(
            lgb.write_to_buffer(Platform::Win32).unwrap(),
            good_lgb_bytes
        );
    }
}
