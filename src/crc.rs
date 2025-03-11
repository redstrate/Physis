// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use libz_rs_sys::z_off_t;
use std::ops::{Add, AddAssign, BitXor, BitXorAssign};

/// CRC used for filepath hashes in index file
pub(crate) struct Jamcrc {
    table: [u32; 256],
}

impl Jamcrc {
    pub(crate) const fn new() -> Self {
        let mut table: [u32; 256] = [0u32; 256];

        let polynomial: u32 = 0xEDB88320;
        let mut i = 0;
        while i < table.len() {
            let mut c: u32 = i as u32;
            let mut j = 0;
            while j < 8 {
                if (c & 1u32) == 1u32 {
                    c = polynomial ^ (c >> 1);
                } else {
                    c >>= 1;
                }
                j += 1;
            }

            table[i] = c;
            i += 1;
        }

        Self { table }
    }

    pub(crate) fn checksum(&self, bytes: &[u8]) -> u32 {
        let mut c: u32 = 0xFFFFFFFF;
        for byte in bytes {
            c = self.table[((c ^ *byte as u32) & 0xFF) as usize] ^ (c >> 8);
        }

        !(c ^ 0xFFFFFFFF)
    }
}

fn crc32(crc: u32, s: &[u8]) -> u32 {
    unsafe { libz_rs_sys::crc32(crc.into(), s.as_ptr(), s.len() as u32) as u32 }
}

fn crc32_combine(crc1: u32, crc2: u32, len2: usize) -> u32 {
    libz_rs_sys::crc32_combine(crc1.into(), crc2.into(), len2 as z_off_t) as u32
}

/// CRC used for shader keys
/// Credit to https://github.com/NotNite/crcracker/ for the original Rust code
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub(crate) struct XivCrc32 {
    pub crc: u32,
    pub len: usize,
}

impl XivCrc32 {
    pub(crate) fn new(crc: u32, len: usize) -> Self {
        Self { crc, len }
    }
}

impl From<&[u8]> for XivCrc32 {
    fn from(s: &[u8]) -> Self {
        Self::new(!crc32(0xFFFFFFFF, s), s.len())
    }
}

impl<const N: usize> From<&[u8; N]> for XivCrc32 {
    fn from(s: &[u8; N]) -> Self {
        Self::new(!crc32(0xFFFFFFFF, s), N)
    }
}

impl From<&str> for XivCrc32 {
    fn from(s: &str) -> Self {
        Self::from(s.as_bytes())
    }
}

impl Add<XivCrc32> for XivCrc32 {
    type Output = XivCrc32;

    fn add(self, rhs: XivCrc32) -> Self::Output {
        Self::new(
            crc32_combine(self.crc, rhs.crc, rhs.len),
            self.len + rhs.len,
        )
    }
}

impl AddAssign<XivCrc32> for XivCrc32 {
    fn add_assign(&mut self, rhs: XivCrc32) {
        self.crc = crc32_combine(self.crc, rhs.crc, rhs.len);
        self.len += rhs.len;
    }
}

impl BitXor<XivCrc32> for XivCrc32 {
    type Output = XivCrc32;

    fn bitxor(self, rhs: XivCrc32) -> Self::Output {
        Self::new(self.crc ^ rhs.crc, self.len.max(rhs.len))
    }
}

impl BitXorAssign<XivCrc32> for XivCrc32 {
    fn bitxor_assign(&mut self, rhs: XivCrc32) {
        self.crc ^= rhs.crc;
        self.len = self.len.max(rhs.len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crc::{Algorithm, Crc};

    #[test]
    fn check_jamcrc() {
        use crc::{CRC_32_JAMCRC, Crc};

        const JAMCR: Crc<u32> = Crc::<u32>::new(&CRC_32_JAMCRC);

        let bytes: [u8; 9] = [1, 1, 2, 4, 5, 6, 12, 12, 12];

        const CRC: Jamcrc = Jamcrc::new();

        assert_eq!(JAMCR.checksum(&bytes), CRC.checksum(&bytes))
    }

    #[test]
    fn check_xivcrc() {
        const CRC_32_TEST: Algorithm<u32> = Algorithm {
            width: 32,
            poly: 0x04c11db7,
            init: 0x00000000,
            refin: true,
            refout: true,
            xorout: 0x00000000,
            check: 0x765e7680,
            residue: 0xc704dd7b,
        };
        const JAMCR: Crc<u32> = Crc::<u32>::new(&CRC_32_TEST);

        let str = "Default";

        assert_eq!(XivCrc32::from(str).crc, JAMCR.checksum(str.as_bytes()));
    }
}
