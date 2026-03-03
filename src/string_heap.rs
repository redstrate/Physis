// SPDX-FileCopyrightText: 2025 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{
    fmt::{Display, Formatter},
    io::{Cursor, Read, Seek, SeekFrom, Write},
};

use binrw::{BinRead, BinReaderExt, BinResult, BinWrite, Endian, Error, VecArgs, binrw};

use crate::{
    ByteBuffer,
    common_file_operations::{read_null_terminated_utf8, write_string},
};

/// A string that exists in a different location in the file, usually a heap with a bunch of other strings.
/// Pointer points to where the string offset is relative to, usually the start of a struct.
#[binrw]
#[br(import(pointer: HeapPointer, string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Clone, PartialEq)]
pub struct HeapString {
    #[br(temp)]
    // TODO: this cast is stupid
    #[bw(calc = string_heap.get_free_offset_string(value) as u32)]
    pub(crate) offset: u32,
    #[br(calc = string_heap.read_string(r, offset.saturating_add(pointer.pos as u32),))]
    #[bw(ignore)]
    pub value: String,
}

impl Display for HeapString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

impl std::fmt::Debug for HeapString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

#[derive(Debug)]
pub struct StringHeap {
    pub(crate) pos: i64,
    pub(crate) bytes: Vec<u8>,
    pub(crate) free_pos: u64,
}

#[binrw]
#[derive(Clone, Copy, Debug)]
pub struct HeapPointer {
    #[br(parse_with = read_pointer_pos)]
    #[bw(ignore)]
    pub(crate) pos: u64,
}

#[binrw::parser(reader)]
pub(crate) fn read_pointer_pos() -> BinResult<u64> {
    Ok(reader.stream_position().unwrap())
}

impl StringHeap {
    pub(crate) fn from(pos: i64) -> Self {
        Self {
            pos,
            bytes: Vec::new(),
            free_pos: 0, // unused, so it doesn't matter
        }
    }

    pub(crate) fn get_free_offset_args<T>(&mut self, obj: &T) -> i32
    where
        T: for<'a> BinWrite<Args<'a> = (&'a mut StringHeap,)> + std::fmt::Debug,
    {
        // NOTE: We have to do this weird "pass a fake data heap" business to recreate what their LGB writer does.
        // I need to explain this better, but basically we can have nested writes to StringHeap that needs to be in a certain order.

        // figure out size of it
        let mut buffer = ByteBuffer::new();
        {
            let mut data_heap = StringHeap {
                pos: 0,
                bytes: Vec::new(),
                free_pos: 0,
            };
            let mut cursor = Cursor::new(&mut buffer);
            obj.write_le_args(&mut cursor, (&mut data_heap,)).unwrap();
        }

        let old_pos = self.free_pos;
        self.free_pos += buffer.len() as u64;

        {
            let mut data_heap = StringHeap {
                pos: buffer.len() as i64,
                bytes: Vec::new(),
                free_pos: buffer.len() as u64,
            };
            buffer.clear();
            let mut cursor = Cursor::new(&mut buffer);
            obj.write_le_args(&mut cursor, (&mut data_heap,)).unwrap();
        }
        self.bytes.append(&mut buffer);
        {
            let mut cursor = Cursor::new(&mut buffer);
            obj.write_le_args(&mut cursor, (self,)).unwrap();
        }

        old_pos as i32
    }

    pub(crate) fn get_free_offset<T>(&mut self, obj: &T) -> i32
    where
        T: for<'a> BinWrite<Args<'a> = ()> + std::fmt::Debug,
    {
        // figure out size of it
        let mut buffer = ByteBuffer::new();
        {
            let mut cursor = Cursor::new(&mut buffer);
            obj.write_le(&mut cursor).unwrap();
        }

        let old_pos = self.free_pos;
        self.free_pos += buffer.len() as u64;
        self.bytes.append(&mut buffer);

        old_pos as i32
    }

    pub(crate) fn get_free_offset_string(&mut self, str: &String) -> i32 {
        let bytes = write_string(str);
        self.get_free_offset(&bytes)
    }

    pub(crate) fn read_args<R, T>(
        &self,
        reader: &mut R,
        endian: Endian,
        heap_pointer: HeapPointer,
        offset: i32,
    ) -> T
    where
        R: Read + Seek,
        T: for<'a> BinRead<Args<'a> = (&'a StringHeap,)>,
    {
        let old_pos = reader.stream_position().unwrap();
        reader
            .seek(SeekFrom::Start(
                (self.pos as i32 + heap_pointer.pos as i32 + offset) as u64,
            ))
            .unwrap();
        let obj = reader.read_type_args::<T>(endian, (self,)).unwrap();
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        obj
    }

    pub(crate) fn read_string<R>(&self, reader: &mut R, offset: u32) -> String
    where
        R: Read + Seek,
    {
        let offset = self.pos + offset as i64;

        let old_pos = reader.stream_position().unwrap();
        reader.seek(SeekFrom::Start(offset as u64)).unwrap();
        let s = read_null_terminated_utf8(reader);
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        s
    }

    pub(crate) fn read_vec_args<R, T>(
        &self,
        reader: &mut R,
        endian: Endian,
        string_heap: &StringHeap,
        heap_pointer: HeapPointer,
        count: usize,
        offset: i32,
    ) -> Vec<T>
    where
        R: Read + Seek + BinReaderExt,
        T: for<'a> BinRead<Args<'a> = (&'a StringHeap,)> + 'static,
    {
        let old_pos = reader.stream_position().unwrap();
        reader
            .seek(SeekFrom::Start(
                (self.pos as i32 + heap_pointer.pos as i32 + offset) as u64,
            ))
            .unwrap();

        let obj: Vec<T> = reader
            .read_type_args(
                endian,
                VecArgs::builder()
                    .count(count)
                    .inner((string_heap,))
                    .finalize(),
            )
            .unwrap();
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        obj
    }
}

impl BinWrite for StringHeap {
    type Args<'a> = ();

    fn write_options<W: Write + Seek>(
        &self,
        writer: &mut W,
        endian: Endian,
        (): Self::Args<'_>,
    ) -> Result<(), Error> {
        self.bytes.write_options(writer, endian, ())?;

        Ok(())
    }
}
