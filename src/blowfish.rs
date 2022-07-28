use std::io::{Cursor, Write};
use crate::blowfish_constants::{BLOWFISH_P, BLOWFISH_S};

const ROUNDS: usize = 16;
const KEYBITS: u32 = 64u32 >> 3;

/// Implementation of the Blowfish block cipher, specialized for encrypting and decrypting SqexArg.
pub struct Blowfish {
    p: [u32; 18],
    s: [[u32; 256]; 4],
}

impl Blowfish {
    /// Initializes a new Blowfish session with a key.
    pub fn new(key: &[u8]) -> Blowfish {
        let mut s = Self {
            p: BLOWFISH_P,
            s: BLOWFISH_S,
        };

        let mut j = 0usize;
        for i in 0..ROUNDS + 2 {
            let mut data = 0u32;
            for _ in 0..4 {
                data = (data << 8) | (key[j as usize] as u32);
                j += 1;

                if j >= (KEYBITS as usize) {
                    j = 0;
                }
            }

            s.p[i] ^= data;
        }

        let mut l = 0u32;
        let mut r = 0u32;

        for i in (0..18).step_by(2) {
            let (l_new, r_new) = s.encrypt_pair(l, r);
            s.p[i] = l_new;
            s.p[i + 1] = r_new;

            l = l_new;
            r = r_new;
        }

        for i in 0..4 {
            for j in (0..256).step_by(2) {
                let (l_new, r_new) = s.encrypt_pair(l, r);
                s.s[i][j] = l_new;
                s.s[i][j + 1] = r_new;

                l = l_new;
                r = r_new;
            }
        }

        s
    }

    /// Encrypts a block of data. If the encryption for any reason fails, returns None.
    pub fn encrypt(&self, data: &[u8]) -> Option<Vec<u8>> {
        let padded_data = Blowfish::pad_buffer(data);

        let mut cursor = Cursor::new(Vec::with_capacity(padded_data.len()));

        for i in (0..padded_data.len()).step_by(8) {
            let l_bytes: [u8; 4] = padded_data[i..i + 4].try_into().ok()?;
            let r_bytes: [u8; 4] = padded_data[i + 4..i + 8].try_into().ok()?;

            let (l, r) = self.encrypt_pair(u32::from_le_bytes(l_bytes), u32::from_le_bytes(r_bytes));

            cursor.write(u32::to_le_bytes(l).as_slice()).ok()?;
            cursor.write(u32::to_le_bytes(r).as_slice()).ok()?;
        }

        Some(cursor.into_inner())
    }

    fn pad_buffer(data: &[u8]) -> Vec<u8> {
        let mut padded_length = data.len();
        if data.len() % 8 != 0 {
            padded_length = data.len() + (8 - (data.len() % 8));
        }

        let mut vec = Vec::with_capacity(padded_length);
        vec.resize(padded_length, 0);
        vec[..data.len()].clone_from_slice(data);

        vec
    }

    /// Decrypts a block of data. If the decryption fails due to buffer overflow issues, will return
    /// None - but this does not indicate that the wrong key was used.
    pub fn decrypt(&self, data: &[u8]) -> Option<Vec<u8>> {
        let padded_data = Blowfish::pad_buffer(data);

        let mut buffer = Vec::with_capacity(padded_data.len());
        let mut cursor = Cursor::new(&mut buffer);

        for i in (0..padded_data.len()).step_by(8) {
            let l_bytes: [u8; 4] = padded_data[i..i + 4].try_into().ok()?;
            let r_bytes: [u8; 4] = padded_data[i + 4..i + 8].try_into().ok()?;

            let (l, r) = self.decrypt_pair(u32::from_le_bytes(l_bytes), u32::from_le_bytes(r_bytes));

            cursor.write(u32::to_le_bytes(l).as_slice()).ok()?;
            cursor.write(u32::to_le_bytes(r).as_slice()).ok()?;
        }

        Some(buffer)
    }

    /// Calculates the F-function for `x`.
    fn f(&self, x: u32) -> u32 {
        let a = self.s[0][(x >> 24) as usize];
        let b = self.s[1][((x >> 16) & 0xFF) as usize];
        let c = self.s[2][((x >> 8) & 0xFF) as usize];
        let d = self.s[3][(x & 0xFF) as usize];

        (a.wrapping_add(b) ^ c).wrapping_add(d)
    }

    fn encrypt_pair(&self, mut l: u32, mut r: u32) -> (u32, u32) {
        for i in (0..ROUNDS).step_by(2) {
            l ^= self.p[i];
            r ^= self.f(l);
            r ^= self.p[i + 1];
            l ^= self.f(r);
        }

        return (r ^ self.p[17], l ^ self.p[16]);
    }

    fn decrypt_pair(&self, mut l: u32, mut r: u32) -> (u32, u32) {
        for i in (2..ROUNDS + 1).step_by(2).rev() {
            l ^= self.p[i + 1];
            r ^= self.f(l);
            r ^= self.p[i];
            l ^= self.f(r);
        }

        return (r ^ self.p[0], l ^ self.p[1]);
    }
}