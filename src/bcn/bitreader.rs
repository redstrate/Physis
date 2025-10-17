// SPDX-FileCopyrightText: 2023 Rudolf Kolbe
// SPDX-License-Identifier: MIT

#[inline]
fn getbits_raw(buf: &[u8], bit_offset: usize, num_bits: usize, dst: &mut [u8]) {
    let bytes_offset = bit_offset / 8;
    let bytes_end: usize = (bit_offset + num_bits).div_ceil(8);
    dst[0..(bytes_end - bytes_offset)].copy_from_slice(&buf[bytes_offset..bytes_end]);
}

pub struct BitReader<'a> {
    data: &'a [u8],
    bit_pos: usize,
}

impl BitReader<'_> {
    #[inline]
    pub const fn new(data: &[u8], bit_pos: usize) -> BitReader<'_> {
        BitReader { data, bit_pos }
    }

    #[inline]
    pub fn read(&mut self, num_bits: usize) -> u16 {
        let ret = self.peek(0, num_bits);
        self.bit_pos += num_bits;
        ret
    }

    #[inline]
    pub fn peek(&self, offset: usize, num_bits: usize) -> u16 {
        let bit_pos = self.bit_pos + offset;
        let shift = bit_pos & 7;

        let mut raw = [0u8; 4];
        getbits_raw(self.data, bit_pos, num_bits, &mut raw);
        let data: u32 = u32::from_le_bytes(raw);

        (data >> shift as u32) as u16 & ((1 << num_bits as u16) - 1)
    }
}
