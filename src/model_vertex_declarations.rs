// SPDX-FileCopyrightText: 2023 Joshua Goins <josh@redstrate.com>
// SPDX-License-Identifier: GPL-3.0-or-later

use std::io::SeekFrom;
use binrw::{BinRead, BinResult, binrw, BinWrite};
use crate::model::NUM_VERTICES;

// Marker for end of stream (0xFF)
const END_OF_STREAM: u8 = 0xFF;

#[binrw]
#[brw(repr = u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VertexType {
    /// 1 32-bit float
    Single1 = 0,
    /// 2 32-bit floats
    Single2 = 1,
    /// 3 32-bit floats
    Single3 = 2,
    /// 4 32-bit floats
    Single4 = 3,

    /// 4 bytes
    Byte4 = 5,

    /// 2 16-bit signed integers
    Short2 = 6,
    /// 4 16-bit signed integers
    Short4 = 7,

    /// 4 8-bit floats
    ByteFloat4 = 8,

    /// Duplicate of Short2?
    Short2n = 9,
    /// Duplicate of Short4?
    Short4n = 10,

    /// 2 16-bit floats
    Half2 = 13,
    /// 4 16-bit floats
    Half4 = 14,

    /// 2 16-bit unsigned integers
    UnsignedShort2 = 16,
    /// 4 16-bit unsigned integers
    UnsignedShort4 = 17
}

#[binrw]
#[brw(repr = u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VertexUsage {
    Position = 0,
    BlendWeights = 1,
    BlendIndices = 2,
    Normal = 3,
    UV = 4,
    Tangent = 5,
    BiTangent = 6,
    Color = 7,
}

#[binrw]
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(dead_code)]
#[brw(little)]
pub struct VertexElement {
    pub stream: u8,
    pub offset: u8,
    pub vertex_type: VertexType,
    pub vertex_usage: VertexUsage,
    #[brw(pad_after = 3)]
    pub usage_index: u8,
}

pub const VERTEX_ELEMENT_SIZE: usize = std::mem::size_of::<VertexElement>() + 3;

#[derive(Clone, Debug, PartialEq)]
pub struct VertexDeclaration {
    pub elements: Vec<VertexElement>,
}

#[binrw::parser(reader, endian)]
pub(crate) fn vertex_element_parser(count: u16) -> BinResult<Vec<VertexDeclaration>> {
    let mut vertex_declarations: Vec<VertexDeclaration> =
        vec![
            VertexDeclaration { elements: vec![] };
            count.into()
        ];
    for declaration in &mut vertex_declarations {
        let mut element = VertexElement::read_options(reader, endian, ())?;

        loop {
            declaration.elements.push(element);

            element = VertexElement::read_options(reader, endian, ())?;

            if element.stream == END_OF_STREAM {
                break;
            }
        }

        let to_seek = NUM_VERTICES as usize * 8 - (declaration.elements.len() + 1) * 8;
        reader.seek(SeekFrom::Current(to_seek as i64))?;
    }

    Ok(vertex_declarations)
}

#[binrw::writer(writer, endian)]
pub(crate) fn vertex_element_writer(
    declarations: &Vec<VertexDeclaration>,
) -> BinResult<()> {
    // write vertex declarations
    for declaration in declarations {
        for element in &declaration.elements {
            element.write_options(writer, endian, ())?;
        }

        VertexElement {
            stream: END_OF_STREAM,
            offset: 0,
            vertex_type: VertexType::Single1,
            vertex_usage: VertexUsage::Position,
            usage_index: 0
        }.write_options(writer, endian, ())?;

        let to_seek = (NUM_VERTICES as usize - 1 - declaration.elements.len()) * 8;
        writer.seek(SeekFrom::Current(to_seek as i64))?;
    }

    Ok(())
}

