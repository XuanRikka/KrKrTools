use binrw::{BinRead, BinWrite};


#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct PsbHeader {
    #[brw(magic = b"PSB\0")]

    #[br(assert(version >= 1 && version <= 4, "unsupported version: {}", version))]
    pub version: u16,

    pub header_encrypt: u16,
    pub header_length: u32,
    pub offset_names: u32,
    pub offset_strings: u32,
    pub offset_strings_data: u32,
    pub offset_chunk_offsets: u32,
    pub offset_chunk_lengths: u32,
    pub offset_chunk_data: u32,
    pub offset_entries: u32,

    // v3+ 字段
    #[br(if(version >= 3))]
    pub checksum: Option<u32>,

    // v4+ 字段
    #[br(if(version >= 4))]
    pub offset_extra_chunk_offsets: Option<u32>,
    #[br(if(version >= 4))]
    pub offset_extra_chunk_lengths: Option<u32>,
    #[br(if(version >= 4))]
    pub offset_extra_chunk_data: Option<u32>,
}