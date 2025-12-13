use std::fs::File;
use std::io::{copy, Cursor, Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use binrw::BinRead;

use crate::units::models::*;
use crate::units::zlib_tool::{decompress, decompress_stream, decompress_stream_vec};


fn file_range(mut file: Arc<File>, start: u64, len: u64) -> impl Read + Seek {
    file.seek(SeekFrom::Start(start)).expect("读取数据失败");
    file.take(len)
}


pub struct XP3Parser
{
    pub file: Arc<File>,
    pub index_offset: u64,
    pub file_index_header: FileIndexHeader,
    pub file_index: Vec<FileIndexEntry>,
    pub file_name_list: Vec<String>,
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

            let decode_data = decompress_stream_vec(&file);
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

        let file_name_list = files.iter().map(|x| x.info.name.clone()).collect::<Vec<_>>();
        
        XP3Parser {
            file: Arc::from(file),
            index_offset,
            file_index_header,
            file_index: files,
            file_name_list,
        }

    }
    pub fn get_file_name(&self) -> Vec<String>
    {
        self.file_name_list.clone()
    }

    pub fn get_file<W: Write>(&mut self, name: &str, mut output: W)
    {
        let entry = match self.file_index.iter().find(|x| x.info.name == name) {
            Some(entry) => entry,
            None => return,
        };

        for i in &entry.segment.segment
        {
            let mut range = file_range(self.file.clone(), i.offset, i.compressed_size);
            if i.flag == 1
            {
                decompress_stream(range, &mut output);
            }
            else
            {
                copy(&mut range, &mut output).expect("读取数据失败");
            };
        };
    }

}
