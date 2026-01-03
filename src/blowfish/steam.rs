// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use super::constants::{BLOWFISH_P, BLOWFISH_S};

const ROUNDS: usize = 16;

/// Implementation of the Blowfish block cipher, specialized for encrypting and decrypting Steam tickets in the launcher. This is somehow *different* than the one used with SqexArg.
pub struct SteamTicketBlowfish {
    p: [u32; 18],
    s: [[u32; 256]; 4],
}

impl SteamTicketBlowfish {
    /// Initializes a new Blowfish session with a key.
    pub fn new(key: &[u8]) -> Self {
        let mut s = Self {
            p: BLOWFISH_P,
            s: BLOWFISH_S,
        };

        let mut j = 0usize;
        for i in 0..ROUNDS + 2 {
            let mut data = 0i32;
            for _ in 0..4 {
                data = (data.wrapping_shl(8)) | ((key[j] as i8) as i32);
                j += 1;

                if j >= key.len() {
                    j = 0;
                }
            }

            s.p[i] ^= data as u32;
        }

        let mut l = 0u32;
        let mut r = 0u32;

        for i in (0..ROUNDS + 2).step_by(2) {
            s.encrypt_pair(&mut l, &mut r);
            s.p[i] = l;
            s.p[i + 1] = r;
        }

        for i in 0..4 {
            for j in (0..256).step_by(2) {
                s.encrypt_pair(&mut l, &mut r);
                s.s[i][j] = l;
                s.s[i][j + 1] = r;
            }
        }

        s
    }

    /// Encrypts a block of data.
    pub fn encrypt(&self, data: &mut [u8]) {
        let padded_size = data.len();

        for i in (0..padded_size).step_by(8) {
            let mut l: u32 = u32::from_be_bytes(data[i..i + 4].try_into().unwrap());
            let mut r: u32 = u32::from_be_bytes(data[i + 4..i + 8].try_into().unwrap());

            self.encrypt_pair(&mut l, &mut r);

            data[i..i + 4].copy_from_slice(&l.to_be_bytes());
            data[i + 4..i + 8].copy_from_slice(&r.to_be_bytes());
        }
    }

    /// Calculates the F-function for `x`.
    fn f(&self, x: u32) -> u32 {
        let [a, b, c, d] = x.to_le_bytes();
        ((self.s[0][d as usize].wrapping_add(self.s[1][c as usize])) ^ (self.s[2][b as usize]))
            .wrapping_add(self.s[3][a as usize])
    }

    fn encrypt_pair(&self, xl: &mut u32, xr: &mut u32) {
        for i in 0..ROUNDS {
            *xl ^= self.p[i];
            *xr ^= self.f(*xl);

            (*xl, *xr) = (*xr, *xl);
        }

        (*xl, *xr) = (*xr, *xl);

        *xr ^= self.p[ROUNDS];
        *xl ^= self.p[ROUNDS + 1];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let blowfish = SteamTicketBlowfish::new(b"00000000#un@e=x>");

        let mut data = [
            8, 187, 54, 57, 54, 52, 55, 53, 54, 56, 54, 97, 55, 53, 54, 49, 54, 52, 51, 56, 51, 57,
            54, 49, 51, 56, 54, 49, 54, 56, 54, 49, 54, 52, 0, 70, 111, 84, 112, 108,
        ]
        .to_vec();

        let expected_encrypted = [
            0x5A, 0xD4, 0x61, 0xAA, 0x1B, 0xD6, 0x7F, 0x1, 0x7, 0xB3, 0xBC, 0xCA, 0x8A, 0x40, 0x2B,
            0xD5, 0xE5, 0xE3, 0x41, 0x8D, 0x26, 0xD5, 0x42, 0x27, 0x13, 0x44, 0x7C, 0x45, 0xF0,
            0x7E, 0xB7, 0x35, 0x2A, 0x6F, 0xF, 0xFB, 0xD, 0xE6, 0x29, 0xC8,
        ];

        blowfish.encrypt(&mut data);

        assert_eq!(data, expected_encrypted);
    }
}
