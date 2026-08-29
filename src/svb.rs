// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;

use crate::ByteBuffer;
use crate::ByteSpan;
use crate::ReadableFile;
use crate::WritableFile;
use crate::common::Platform;
use binrw::BinRead;
use binrw::BinWrite;
use binrw::binrw;

/// Sky visibility binary file, usually with the `.svb` file extension.
#[binrw]
#[derive(Debug)]
#[brw(magic = b"SVB1")]
pub struct Svb {
    /// Including this header, in bytes.
    #[bw(calc = self.calculate_file_size())]
    file_size: u32,
    /// Number of Svc's
    #[br(temp)]
    #[bw(calc = svcs.len() as u32)]
    svc_count: u32,
    #[br(count = svc_count)]
    pub svcs: Vec<Svc>,
}

impl Svb {
    pub(crate) const HEADER_SIZE: u32 = 12;
}

#[binrw]
#[derive(Debug)]
#[brw(magic = b"SVC1")]
pub struct Svc {
    /// In bytes, including the magic.
    #[bw(calc = Self::HEADER_SIZE)]
    header_size: u32,
    /// Seems to always be 0?
    id: u32,
    /// Always seems to be 12?
    unk1: u32,

    #[br(temp)]
    #[bw(calc = entries.len() as u32)]
    pub num_entries: u32,
    #[br(count = num_entries)]
    pub entries: Vec<SvcEntry>,
}

impl Svc {
    pub const HEADER_SIZE: u32 = 20;

    fn calculate_file_size(&self) -> u32 {
        Self::HEADER_SIZE + (self.entries.len() as u32) * SvcEntry::SIZE
    }
}

#[binrw]
#[derive(Debug)]
pub struct SvcEntry {
    /// Points to a GameObject in this territory.
    pub instance_id: u32,
    pub members: [u8; 4],
    pub visibility: f32,
}

impl SvcEntry {
    const SIZE: u32 = 12;
}

impl ReadableFile for Svb {
    fn from_existing(platform: Platform, buffer: ByteSpan) -> crate::Result<Self> {
        let mut cursor = Cursor::new(buffer);
        Ok(Self::read_options(&mut cursor, platform.endianness(), ())?)
    }
}

impl WritableFile for Svb {
    fn write_to_buffer(&self, platform: Platform) -> crate::Result<ByteBuffer> {
        let mut buffer = ByteBuffer::new();

        {
            let mut cursor = Cursor::new(&mut buffer);
            self.write_options(&mut cursor, platform.endianness(), ())?;
        }

        Ok(buffer)
    }
}

impl Svb {
    /// Creates an empty SVB.
    pub fn new() -> Self {
        Self {
            svcs: vec![Svc {
                id: 0,
                unk1: 12,
                entries: Vec::new(),
            }],
        }
    }

    fn calculate_file_size(&self) -> u32 {
        Svb::HEADER_SIZE
            + self
                .svcs
                .iter()
                .map(|x| x.calculate_file_size())
                .sum::<u32>()
    }
}

impl Default for Svb {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::read, path::PathBuf};

    use crate::pass_random_invalid;

    use super::*;

    #[test]
    fn test_invalid() {
        pass_random_invalid::<Svb>();
    }

    #[test]
    fn test_write_empty() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("empty.svb");

        let empty_svb = &read(d).unwrap();
        let svb = Svb::new();

        assert_eq!(*empty_svb, svb.write_to_buffer(Platform::Win32).unwrap());
    }

    #[test]
    fn test_simple() {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/tests");
        d.push("s1e6.svb");

        let file = &read(d).unwrap();
        let svb = Svb {
            svcs: vec![Svc {
                id: 0,
                unk1: 12,
                entries: vec![
                    SvcEntry {
                        instance_id: 2378523,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378521,
                        members: [0, 0, 0, 0],
                        visibility: 1.0,
                    },
                    SvcEntry {
                        instance_id: 2378520,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378524,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378528,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378527,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378526,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378516,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378542,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378525,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378522,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378543,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378515,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378536,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378544,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378540,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378539,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378538,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378541,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 4222649,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378518,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378517,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378537,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378531,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378530,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378529,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378532,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378535,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378534,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                    SvcEntry {
                        instance_id: 2378533,
                        members: [0, 0, 0, 0],
                        visibility: 0.25,
                    },
                ],
            }],
        };

        // round-trip
        assert_eq!(*file, svb.write_to_buffer(Platform::Win32).unwrap());
    }
}
