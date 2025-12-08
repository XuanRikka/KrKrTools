use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::process::exit;
use binrw::BinRead;

use crate::units::models::*;
use crate::units::zlib_tool::decompress;



pub struct XP3Parser
{
    pub file: File,
    pub index_offset: u64,
    pub file_index_header: FileIndexHeader,
    pub file_index: Vec<FileIndexEntry>,
}


impl XP3Parser
{
    pub fn new(mut file: File) -> Self
    {
        // 验证magic
        let mut magic: [u8; 11] = [0u8; 11];
        file.read_exact(&mut magic).expect("读取文件失败");
        if magic != [0x58,0x50,0x33,0x0d,0x0a,0x20,0x0a,0x1a,0x8b,0x67,0x01]
        {
            panic!("不是XP3文件");
        }

        // 检测是是否存在缓冲区并seek
        let mut v230header: [u8; 8] = [0u8; 8];
        file.read_exact(&mut v230header).expect("读取文件失败");
        if v230header != [0x17, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        {
            file.seek(SeekFrom::Start(11)).expect("读取文件失败");
        }
        else
        {
            file.seek(SeekFrom::Start(32)).expect("读取文件失败");
        }

        // 读取目录的绝对偏移
        let mut index_offset: [u8; 8] = [0u8; 8];
        file.read_exact(&mut index_offset).expect("读取文件失败");
        let index_offset = u64::from_le_bytes(index_offset);
        file.seek(SeekFrom::Start(index_offset)).expect("读取文件失败");
        
        let file_index_header = FileIndexHeader::read(&mut file).expect("解析文件目录头失败");

        // 读取全部目录
        let mut files: Vec<FileIndexEntry> = Vec::new();
        if file_index_header.compression_flag == 0x00
        {
            loop {
                let current_pos = file.stream_position().expect("文件读取失败");
                if file.seek(SeekFrom::End(0)).expect("文件读取失败") == current_pos {
                    break;
                }
                file.seek(SeekFrom::Start(current_pos)).expect("文件读取失败");

                match FileIndexEntry::read(&mut file) {
                    Ok(entry) => files.push(entry),
                    Err(binrw::Error::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => panic!("数据解析失败：{:?}", e),
                }
            }
        }
        else
        {
            let mut data: Vec<u8> = Vec::new();
            file.read_to_end(&mut data).expect("读取文件失败");
            let decode_data = decompress(data.as_slice());
            drop(data);
            let mut data_cur = Cursor::new(decode_data);
            loop {
                let current_pos = data_cur.stream_position().expect("文件读取失败");
                if data_cur.seek(SeekFrom::End(0)).expect("文件读取失败") == current_pos {
                    break;
                }
                data_cur.seek(SeekFrom::Start(current_pos)).expect("文件读取失败");

                match FileIndexEntry::read(&mut data_cur) {
                    Ok(entry) => files.push(entry),
                    Err(binrw::Error::Io(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                    Err(e) => panic!("数据解析失败：{:?}", e),
                }
            }
        }


        XP3Parser {
            file,
            index_offset,
            file_index_header,
            file_index: files,
        }

    }
    pub fn get_file_name(self: &Self) -> Vec<String>
    {
        self.file_index.iter().map(|x| x.info.name.clone()).collect()
    }

    pub fn get_file(&mut self, name: String) -> Vec<u8>
    {
        let offsets = self.file_index.iter()
            .filter_map(|x| {
                if x.info.name == name {
                    Some(x.segment.segment.iter().clone())
                } else {
                    None
                }
            })
            .next();
        if !offsets.is_some()
        {
            return Vec::new();
        };


        let mut data: Vec<u8> = Vec::new();
        for i in offsets.unwrap()
        {
            let mut temp_data: Vec<u8> = vec![0u8; i.compressed_size as usize];
            self.file.seek(SeekFrom::Start(i.offset)).expect("读取文件失败");
            self.file.read_exact(&mut *temp_data).expect("读取文件失败");
            if i.flag == 1
            {
                data.extend(decompress(temp_data.as_slice()));
            }
            else
            {
                data.extend(temp_data);
            };
        };

        data
    }

}
