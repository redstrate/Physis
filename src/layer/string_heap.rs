use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use binrw::{BinRead, BinReaderExt, BinWrite, Endian, Error, binrw};

use crate::{ByteBuffer, common_file_operations::write_string};

/// A string that exists in a different location in the file, usually a heap with a bunch of other strings.
#[binrw]
#[br(import(string_heap: &StringHeap), stream = r)]
#[bw(import(string_heap: &mut StringHeap))]
#[derive(Clone, Debug, PartialEq)]
pub struct HeapString {
    #[br(temp)]
    // TODO: this cast is stupid
    #[bw(calc = string_heap.get_free_offset_string(value) as u32)]
    pub offset: u32,
    #[br(calc = string_heap.read_string(r, offset,))]
    #[bw(ignore)]
    pub value: String,
}

#[derive(Debug)]
pub struct StringHeap {
    pub pos: u64,
    pub bytes: Vec<u8>,
    pub free_pos: u64,
}

impl StringHeap {
    pub fn from(pos: u64) -> Self {
        Self {
            pos,
            bytes: Vec::new(),
            free_pos: 0, // unused, so it doesn't matter
        }
    }

    pub fn get_free_offset_args<T>(&mut self, obj: &T) -> i32
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
                pos: buffer.len() as u64,
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

    pub fn get_free_offset<T>(&mut self, obj: &T) -> i32
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

    pub fn get_free_offset_string(&mut self, str: &String) -> i32 {
        let bytes = write_string(str);
        self.get_free_offset(&bytes)
    }

    pub fn read<R, T>(&self, reader: &mut R, offset: i32) -> T
    where
        R: Read + Seek,
        T: for<'a> BinRead<Args<'a> = ()>,
    {
        let old_pos = reader.stream_position().unwrap();
        reader
            .seek(SeekFrom::Start((self.pos as i32 + offset) as u64))
            .unwrap();
        let obj = reader.read_le::<T>().unwrap();
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        obj
    }

    pub fn read_args<R, T>(&self, reader: &mut R, offset: i32) -> T
    where
        R: Read + Seek,
        T: for<'a> BinRead<Args<'a> = (&'a StringHeap,)>,
    {
        let old_pos = reader.stream_position().unwrap();
        reader
            .seek(SeekFrom::Start((self.pos as i32 + offset) as u64))
            .unwrap();
        let obj = reader.read_le_args::<T>((self,)).unwrap();
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        obj
    }

    pub fn read_string<R>(&self, reader: &mut R, offset: u32) -> String
    where
        R: Read + Seek,
    {
        let offset = self.pos + offset as u64;

        let mut string = String::new();

        let old_pos = reader.stream_position().unwrap();

        reader.seek(SeekFrom::Start(offset)).unwrap();
        let mut next_char = reader.read_le::<u8>().unwrap() as char;
        while next_char != '\0' {
            string.push(next_char);
            next_char = reader.read_le::<u8>().unwrap() as char;
        }
        reader.seek(SeekFrom::Start(old_pos)).unwrap();
        string
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
