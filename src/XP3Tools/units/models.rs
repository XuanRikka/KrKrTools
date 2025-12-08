use binrw::{BinRead, BinWrite};

use crate::units::encode_tool::{utf16le_to_string, string_to_utf16le};


pub const V230MAGIC: [u8; 32] = [
    0x58, 0x50, 0x33, 0x0D, 0x0A, 0x20, 0x0A, 0x1A, 0x8B, 0x67, 0x01, 0x17, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
];


#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct FileIndexHeader
{
    pub compression_flag: u8,
    pub compression_size: u64,

    #[br(if(compression_flag != 0x00))]
    pub raw_size: Option<u64>,
}


#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct FileIndexEntry
{
    pub file: FileIndexFile,
    pub info: FileIndexInfo,
    pub segment: FileIndexSegment,
    pub adler32: FileIndexAdler32
}


#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct FileIndexFile
{
    #[brw(magic = b"File")]
    pub entry_size: u64
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct FileIndexInfo
{
    #[brw(magic = b"info")]
    pub entry_size: u64,
    pub flag: u32,
    pub raw_size: u64,
    pub compressed_size: u64,
    pub name_length: u16,

    #[br(map = |bytes: Vec<u16>| utf16le_to_string(&bytes))]
    #[bw(map = |s: &String| string_to_utf16le(s.to_string()))]
    #[br(count = name_length)]
    pub name: String,
}

#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct FileIndexSegment
{
    #[brw(magic = b"segm")]
    pub segment_size: u64,

    #[br(count = segment_size/28)]
    pub segment: Vec<FileIndexSegmentEntry>
}


#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct FileIndexSegmentEntry
{
    pub flag: u32,
    pub offset: u64,
    pub raw_size: u64,
    pub compressed_size: u64
}


#[derive(BinRead, BinWrite, Debug)]
#[brw(little)]
pub struct FileIndexAdler32
{
    #[brw(magic = b"adlr\x04\0\0\0\0\0\0\0")]
    pub adler32: u32,
}

