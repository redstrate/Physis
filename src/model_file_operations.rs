// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::Cursor;
use binrw::{BinReaderExt, BinResult, BinWriterExt};
use half::f16;
use crate::ByteSpan;
use crate::model::MDL;

// Maximum value of byte, used to divide and multiply floats in that space [0.0..1.0] to [0..255]
const MAX_BYTE_FLOAT: f32 = u8::MAX as f32;

impl MDL {
    pub(crate) fn read_byte_float4(cursor: &mut Cursor<ByteSpan>) -> Option<[f32; 4]> {
        Some([
            (f32::from(cursor.read_le::<u8>().ok()?) / MAX_BYTE_FLOAT),
            (f32::from(cursor.read_le::<u8>().ok()?) / MAX_BYTE_FLOAT),
            (f32::from(cursor.read_le::<u8>().ok()?) / MAX_BYTE_FLOAT),
            (f32::from(cursor.read_le::<u8>().ok()?) / MAX_BYTE_FLOAT)
        ])
    }

    pub(crate) fn write_byte_float4<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 4]) -> BinResult<()> {
        cursor.write_le::<[u8; 4]>(&[
            (vec[0] * MAX_BYTE_FLOAT).round() as u8,
            (vec[1] * MAX_BYTE_FLOAT).round() as u8,
            (vec[2] * MAX_BYTE_FLOAT).round() as u8,
            (vec[3] * MAX_BYTE_FLOAT).round() as u8])
    }

    pub(crate) fn read_tangent(cursor: &mut Cursor<ByteSpan>) -> Option<[f32; 4]> {
        Some([
            (f32::from(cursor.read_le::<u8>().ok()?) * 2.0 / MAX_BYTE_FLOAT - 1.0),
            (f32::from(cursor.read_le::<u8>().ok()?) * 2.0 / MAX_BYTE_FLOAT - 1.0),
            (f32::from(cursor.read_le::<u8>().ok()?) * 2.0 / MAX_BYTE_FLOAT - 1.0),
            if (f32::from(cursor.read_le::<u8>().ok()?) * 2.0 / MAX_BYTE_FLOAT - 1.0) == 1.0 { 1.0 } else { -1.0 }
        ])
    }

    pub(crate) fn write_tangent<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 4]) -> BinResult<()> {
        cursor.write_le::<[u8; 4]>(&[
            ((vec[0] + 1.0) * (MAX_BYTE_FLOAT / 2.0)).round() as u8,
            ((vec[1] + 1.0) * (MAX_BYTE_FLOAT / 2.0)).round() as u8,
            ((vec[2] + 1.0) * (MAX_BYTE_FLOAT / 2.0)).round() as u8,
            if vec[3] > 0.0 { 255 } else { 0 }]) // SqEx uses 0 as -1, not 1
    }

    pub(crate) fn read_half4(cursor: &mut Cursor<ByteSpan>) -> Option<[f32; 4]> {
        Some([
            f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32(),
            f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32(),
            f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32(),
            f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32()
        ])
    }

    pub(crate) fn write_half4<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 4]) -> BinResult<()> {
        cursor.write_le::<[u16; 4]>(&[
            f16::from_f32(vec[0]).to_bits(),
            f16::from_f32(vec[1]).to_bits(),
            f16::from_f32(vec[2]).to_bits(),
            f16::from_f32(vec[3]).to_bits()])
    }

    pub(crate) fn read_half2(cursor: &mut Cursor<ByteSpan>) -> Option<[f32; 2]> {
        Some([
            f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32(),
            f16::from_bits(cursor.read_le::<u16>().ok()?).to_f32()
        ])
    }

    pub(crate) fn write_half2<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 2]) -> BinResult<()> {
        cursor.write_le::<[u16; 2]>(&[
            f16::from_f32(vec[0]).to_bits(),
            f16::from_f32(vec[1]).to_bits()])
    }

    pub(crate) fn read_uint(cursor: &mut Cursor<ByteSpan>) -> BinResult<[u8; 4]> {
        cursor.read_le::<[u8; 4]>()
    }

    pub(crate) fn write_uint<T: BinWriterExt>(cursor: &mut T, vec: &[u8; 4]) -> BinResult<()> {
        cursor.write_le::<[u8; 4]>(vec)
    }

    pub(crate) fn read_single3(cursor: &mut Cursor<ByteSpan>) -> BinResult<[f32; 3]> {
        cursor.read_le::<[f32; 3]>()
    }

    pub(crate) fn write_single3<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 3]) -> BinResult<()> {
        cursor.write_le::<[f32; 3]>(vec)
    }

    pub(crate) fn read_single4(cursor: &mut Cursor<ByteSpan>) -> BinResult<[f32; 4]> {
        cursor.read_le::<[f32; 4]>()
    }

    pub(crate) fn write_single4<T: BinWriterExt>(cursor: &mut T, vec: &[f32; 4]) -> BinResult<()> {
        cursor.write_le::<[f32; 4]>(vec)
    }

    pub(crate) fn pad_slice<const N: usize>(small_slice: &[f32; N], fill: f32) -> [f32; 4] {
        let mut bigger_slice: [f32; 4] = [fill, fill, fill, fill];
        bigger_slice[..N].copy_from_slice(&small_slice[..N]);
        bigger_slice
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::model::MDL;

    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            for i in 0..4 {
                if !($x[i] - $y[i] < $d || $y[i] - $x[i] < $d) { panic!(); }
            }
        }
    }

    #[test]
    fn byte_float4() {
        let a = [0.0, 1.0, 0.5, 0.25];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_byte_float4(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());

        let b = MDL::read_byte_float4(&mut read_cursor).unwrap();
        assert_delta!(b, a, 0.1);
    }

    #[test]
    fn half4() {
        let a = [0.0, 1.0, 0.5, 0.25];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_half4(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_half4(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn half2() {
        let a = [0.0, 1.0];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_half2(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_half2(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn uint() {
        let a = [5u8, 0u8, 3u8, 15u8];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_uint(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_uint(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn single3() {
        let a = [3.0, 0.0, -1.0];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_single3(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_single3(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn single4() {
        let a = [3.0, 0.0, -1.0, 12.0];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_single4(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        assert_eq!(MDL::read_single4(&mut read_cursor).unwrap(), a);
    }

    #[test]
    fn tangent() {
        let a = [1.0, 0.5, -0.5, 1.0];

        let mut v = vec![];
        let mut cursor = Cursor::new(&mut v);

        MDL::write_tangent(&mut cursor, &a).unwrap();

        let mut read_cursor = Cursor::new(v.as_slice());
        let tangent = MDL::read_tangent(&mut read_cursor).unwrap();
        assert_delta!(tangent, a, 0.001);
    }

    #[test]
    fn pad_slice() {
        let a = [3.0, 0.0, -1.0];
        let b = [3.0, 0.0, -1.0, 1.0];

        assert_eq!(MDL::pad_slice(&a, 1.0), b);
    }
}
