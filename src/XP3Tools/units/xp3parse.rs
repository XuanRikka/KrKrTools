use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

use crate::units::constant::V230HEADER;
use crate::units::models::*;
use crate::units::zlib_tool::decompress;

pub struct XP3Parser
{
    index_offset: u64,
    file_index_header: FileIndexHeader,
}


impl XP3Parser
{
    fn new(mut file: BufReader<File>)
    {
        let mut header = [0u8; 11];
        file.read_exact(&mut header).expect("读取文件失败");
        MagicHeader::parser(header.as_slice()).expect("解析头部失败");


        // 读取索引偏移量
        let buf = file.fill_buf().expect("读取文件失败");
        if buf.len() < 8 {
            panic!("文件长度不足");
        }
        let mut v230_header = [0u8; 8];
        v230_header.copy_from_slice(&buf[..8]);
        if v230_header == V230HEADER {
            file.consume(21);
        }
        let mut index_offset_u8 = [0u8;8];
        file.read_exact(&mut index_offset_u8).expect("读取文件失败");
        let index_offset = u64::from_le_bytes(index_offset_u8);

        // 读取索引头部
        file.seek(SeekFrom::Start(index_offset)).expect("读取文件失败");
        let mut index_header_u8 = [0u8, 13];
        file.read_exact(&mut index_header_u8).expect("读取文件失败");
        let file_index_header = FileIndexHeader::parser(index_header_u8.as_slice()).expect("解析索引头部失败");


        // 读取文件索引


    }
}
