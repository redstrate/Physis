use crate::common::Language;
use crate::gamedata::MemoryBuffer;
use binrw::binread;
use binrw::BinRead;
use std::io::Cursor;

#[binread]
#[br(magic = b"EXHF")]
#[br(big)]
pub struct EXHHeader {
    version: u16,

    pub(crate) data_offset: u16,
    column_count: u16,
    page_count: u16,
    language_count: u16,

    #[br(pad_before = 6)]
    #[br(pad_after = 8)]
    pub(crate) row_count: u32,
}

#[binread]
#[br(repr(u16))]
pub enum ColumnDataType {
    String = 0x0,
    Bool = 0x1,
    Int8 = 0x2,
    UInt8 = 0x3,
    Int16 = 0x4,
    UInt16 = 0x5,
    Int32 = 0x6,
    UInt32 = 0x7,
    Float32 = 0x9,
    Int64 = 0xA,
    UInt64 = 0xB,

    PackedBool0 = 0x19,
    PackedBool1 = 0x1A,
    PackedBool2 = 0x1B,
    PackedBool3 = 0x1C,
    PackedBool4 = 0x1D,
    PackedBool5 = 0x1E,
    PackedBool6 = 0x1F,
    PackedBool7 = 0x20,
}

#[binread]
#[br(big)]
pub struct ExcelColumnDefinition {
    pub data_type: ColumnDataType,
    pub offset: u16,
}

#[binread]
#[br(big)]
pub struct ExcelDataPagination {
    pub start_id: u32,
    row_count: u32,
}

#[binread]
#[br(big)]
pub struct EXH {
    pub header: EXHHeader,

    #[br(count = header.column_count)]
    pub column_definitions: Vec<ExcelColumnDefinition>,

    #[br(count = header.page_count)]
    pub pages: Vec<ExcelDataPagination>,

    #[br(count = header.language_count)]
    languages: Vec<Language>,
}

impl EXH {
    pub fn from_existing(buffer: &MemoryBuffer) -> Option<EXH> {
        EXH::read(&mut Cursor::new(&buffer)).ok()
    }
}
