// SPDX-FileCopyrightText: 2024 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::io::{Cursor, Read, Seek, SeekFrom};

use crate::common::Platform;
use crate::common_file_operations::{Half1, Half3};
use crate::{ByteSpan, ReadableFile};
use binrw::BinReaderExt;

/// A single STM entry containing dye color data for all dye indices.
#[derive(Debug, Clone)]
pub struct StmEntry {
    pub diffuse: Vec<[f32; 3]>,
    pub specular: Vec<[f32; 3]>,
    pub emissive: Vec<[f32; 3]>,
    pub gloss: Vec<f32>,
    pub specular_power: Vec<f32>,
}

impl StmEntry {
    /// Get diffuse color for a given stain index (0-based, so stain_id - 1).
    pub fn get_diffuse(&self, stain_index: usize) -> Option<[f32; 3]> {
        self.diffuse.get(stain_index).copied()
    }

    /// Get specular color for a given stain index (0-based).
    pub fn get_specular(&self, stain_index: usize) -> Option<[f32; 3]> {
        self.specular.get(stain_index).copied()
    }

    /// Get emissive color for a given stain index (0-based).
    pub fn get_emissive(&self, stain_index: usize) -> Option<[f32; 3]> {
        self.emissive.get(stain_index).copied()
    }

    /// Get gloss value for a given stain index (0-based).
    pub fn get_gloss(&self, stain_index: usize) -> Option<f32> {
        self.gloss.get(stain_index).copied()
    }

    /// Get specular power value for a given stain index (0-based).
    pub fn get_specular_power(&self, stain_index: usize) -> Option<f32> {
        self.specular_power.get(stain_index).copied()
    }
}

/// Dye pack with all color components for a single stain.
#[derive(Debug, Clone)]
pub struct DyePack {
    pub diffuse: [f32; 3],
    pub specular: [f32; 3],
    pub emissive: [f32; 3],
    pub gloss: f32,
    pub specular_power: f32,
}

/// Staining template material file, usually with the `.stm` file extension.
///
/// Contains dye color information indexed by template ID and stain index.
/// Supports both old format (u16 keys, 128 dyes) and new format (u32 keys, 254 dyes).
#[derive(Debug)]
pub struct StainingTemplate {
    pub entries: HashMap<u16, StmEntry>,
}

impl StainingTemplate {
    /// Look up a complete DyePack for a given template ID and stain index (0-based).
    ///
    /// Handles Dawntrail template ID mapping: IDs >= 1000 are mapped to (id - 1000)
    /// in the legacy STM file.
    pub fn get_dye_pack(&self, template_id: u16, stain_index: usize) -> Option<DyePack> {
        // Dawntrail templates (>= 1000) map to legacy templates by stripping the prefix
        let key = if template_id >= 1000 {
            template_id - 1000
        } else {
            template_id
        };
        let entry = self.entries.get(&key)?;
        Some(DyePack {
            diffuse: entry.get_diffuse(stain_index).unwrap_or([1.0, 1.0, 1.0]),
            specular: entry.get_specular(stain_index).unwrap_or([1.0, 1.0, 1.0]),
            emissive: entry.get_emissive(stain_index).unwrap_or([0.0, 0.0, 0.0]),
            gloss: entry.get_gloss(stain_index).unwrap_or(0.0),
            specular_power: entry.get_specular_power(stain_index).unwrap_or(0.0),
        })
    }

    /// Read an array of Half3 values and convert to Vec<[f32; 3]>.
    fn read_half3_array(
        cursor: &mut Cursor<ByteSpan>,
        offset: u64,
        size: usize,
        num_dyes: usize,
    ) -> Vec<[f32; 3]> {
        let raw: Vec<Half3> = Self::read_array::<Half3>(cursor, offset, size, num_dyes);
        raw.iter()
            .map(|h| [h.r.to_f32(), h.g.to_f32(), h.b.to_f32()])
            .collect()
    }

    /// Read an array of Half1 values and convert to Vec<f32>.
    fn read_half1_array(
        cursor: &mut Cursor<ByteSpan>,
        offset: u64,
        size: usize,
        num_dyes: usize,
    ) -> Vec<f32> {
        let raw: Vec<Half1> = Self::read_array::<Half1>(cursor, offset, size, num_dyes);
        raw.iter().map(|h| h.value.to_f32()).collect()
    }

    /// Read a sub-table, detecting the encoding mode:
    ///
    /// - **Singleton** (array_size == 1): single value replicated for all dyes
    /// - **OneToOne** (array_size >= num_dyes): direct values, one per dye
    /// - **Indexed** (1 < array_size < num_dyes): palette + marker byte + index table
    ///   - palette_count = (size - num_dyes) / sizeof(T)
    ///   - First byte of index section is a marker (0xFF), skipped
    ///   - Indices are 1-based: 0 or 255 → default, else → palette[index - 1]
    ///   - Last dye entry is forced to default
    fn read_array<T: binrw::BinRead<Args<'static> = ()> + Default + Clone + Copy>(
        cursor: &mut Cursor<ByteSpan>,
        offset: u64,
        size: usize,
        num_dyes: usize,
    ) -> Vec<T> {
        let elem_size = std::mem::size_of::<T>();
        if elem_size == 0 || size == 0 {
            return vec![T::default(); num_dyes];
        }

        let array_size = size / elem_size;

        if array_size == 0 {
            return vec![T::default(); num_dyes];
        }

        cursor.seek(SeekFrom::Start(offset)).unwrap();

        if array_size == 1 {
            // Singleton: replicate single value for all dyes
            let element = cursor.read_le::<T>().unwrap();
            return vec![element; num_dyes];
        }

        if array_size >= num_dyes {
            // OneToOne: read num_dyes values directly
            let mut result = Vec::with_capacity(num_dyes);
            for _ in 0..num_dyes {
                result.push(cursor.read_le::<T>().unwrap());
            }
            return result;
        }

        // Indexed: palette + marker byte + (num_dyes - 1) index bytes
        // palette_count = (size - num_dyes) / elem_size
        if size < num_dyes {
            return vec![T::default(); num_dyes];
        }
        let palette_count = (size - num_dyes) / elem_size;
        if palette_count == 0 {
            return vec![T::default(); num_dyes];
        }

        // Read palette values
        let mut palette: Vec<T> = Vec::with_capacity(palette_count);
        for _ in 0..palette_count {
            palette.push(cursor.read_le::<T>().unwrap());
        }

        // Read index section (num_dyes bytes: 1 marker + (num_dyes - 1) actual indices)
        let mut index_bytes = vec![0u8; num_dyes];
        cursor.read_exact(&mut index_bytes).unwrap_or(());

        // Build result: skip byte 0 (marker), read bytes 1..(num_dyes-1), last entry = default
        let mut result = Vec::with_capacity(num_dyes);
        for i in 0..num_dyes {
            if i == num_dyes - 1 {
                // Last dye entry is forced to default
                result.push(T::default());
            } else {
                // Index bytes are at positions 1..(num_dyes-1), so read index_bytes[i + 1]
                let index = index_bytes[i + 1] as usize;
                if index == 0 || index == 255 {
                    result.push(T::default());
                } else if index - 1 < palette.len() {
                    result.push(palette[index - 1]);
                } else {
                    result.push(T::default());
                }
            }
        }
        result
    }
}

impl ReadableFile for StainingTemplate {
    fn from_existing(_platform: Platform, buffer: ByteSpan) -> Option<Self> {
        if buffer.len() < 8 {
            return None;
        }

        let mut cursor = Cursor::new(buffer);

        // Header: 4 × u16
        let _magic: u16 = cursor.read_le().ok()?;
        let version: u16 = cursor.read_le().ok()?;
        let entry_count: u16 = cursor.read_le().ok()?;
        let _unknown: u16 = cursor.read_le().ok()?;

        if entry_count == 0 {
            return Some(StainingTemplate {
                entries: HashMap::new(),
            });
        }

        let n = entry_count as usize;

        // Detect old vs new format (matching TexTools heuristic)
        // Old format: u16 keys/offsets, 128 dyes
        // New format: u32 keys/offsets, 254 dyes
        let old_format = if buffer.len() > 0x0B {
            // For Endwalker STM: if the 3rd/4th bytes of the first key entry are non-zero,
            // the keys are u16 (old format). If zero, they're u32 (new format).
            buffer[0x0A] != 0x00 || buffer[0x0B] != 0x00
        } else {
            version < 0x0101
        };

        let num_dyes: usize = if old_format { 128 } else { 254 };

        // Read keys and offsets
        let mut keys = Vec::with_capacity(n);
        let mut offsets = Vec::with_capacity(n);

        if old_format {
            for _ in 0..n {
                keys.push(cursor.read_le::<u16>().ok()? as u32);
            }
            for _ in 0..n {
                offsets.push(cursor.read_le::<u16>().ok()? as u32);
            }
        } else {
            for _ in 0..n {
                keys.push(cursor.read_le::<u32>().ok()?);
            }
            for _ in 0..n {
                offsets.push(cursor.read_le::<u32>().ok()?);
            }
        }

        // data_base = end of header
        let header_entry_size: usize = if old_format { 4 } else { 8 }; // per entry: key + offset
        let end_of_header = 8 + header_entry_size * n;

        let mut entries = HashMap::new();

        for i in 0..n {
            let key = keys[i] as u16;
            let entry_start = offsets[i] as usize * 2 + end_of_header;

            if entry_start + 10 > buffer.len() {
                continue;
            }

            cursor.seek(SeekFrom::Start(entry_start as u64)).ok()?;

            // Read 5 sub-table end offsets (cumulative, in half-word units)
            // Multiply by 2 to get byte offsets
            let mut ends = [0u16; 5];
            for end in &mut ends {
                *end = cursor.read_le::<u16>().ok()?;
            }

            let data_start = entry_start + 10; // 5 × u16 = 10 bytes

            // Compute sub-table byte ranges from cumulative ends
            // ends[i] is in half-word units; multiply by 2 for bytes
            let sub_ranges: [(usize, usize); 5] = {
                let e0 = ends[0] as usize * 2;
                let e1 = ends[1] as usize * 2;
                let e2 = ends[2] as usize * 2;
                let e3 = ends[3] as usize * 2;
                let e4 = ends[4] as usize * 2;
                [
                    (0, e0),           // diffuse
                    (e0, e1 - e0),     // specular
                    (e1, e2 - e1),     // emissive
                    (e2, e3 - e2),     // gloss
                    (e3, e4 - e3),     // specular_power
                ]
            };

            let diffuse = Self::read_half3_array(
                &mut cursor,
                (data_start + sub_ranges[0].0) as u64,
                sub_ranges[0].1,
                num_dyes,
            );
            let specular = Self::read_half3_array(
                &mut cursor,
                (data_start + sub_ranges[1].0) as u64,
                sub_ranges[1].1,
                num_dyes,
            );
            let emissive = Self::read_half3_array(
                &mut cursor,
                (data_start + sub_ranges[2].0) as u64,
                sub_ranges[2].1,
                num_dyes,
            );
            let gloss = Self::read_half1_array(
                &mut cursor,
                (data_start + sub_ranges[3].0) as u64,
                sub_ranges[3].1,
                num_dyes,
            );
            let specular_power = Self::read_half1_array(
                &mut cursor,
                (data_start + sub_ranges[4].0) as u64,
                sub_ranges[4].1,
                num_dyes,
            );

            entries.insert(
                key,
                StmEntry {
                    diffuse,
                    specular,
                    emissive,
                    gloss,
                    specular_power,
                },
            );
        }

        Some(StainingTemplate { entries })
    }
}

#[cfg(test)]
mod tests {
    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<StainingTemplate>();
    }
}
