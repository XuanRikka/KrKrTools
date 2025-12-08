mod units;
use units::path_tool::*;
use units::models::*;
use units::zlib_tool::{compress_stream, compress};

use std::path::{Path, PathBuf};
use std::fs::{File};
use std::hash::Hasher;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use adler::Adler32;
use binrw::BinWrite;
use clap::Parser;
use clap;

#[derive(Parser)]
#[command(name = "XP3Pack")]
#[command(about = "用于打包xp3", version = "1.0")]
struct Cli {
    input: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    no_compress_file: bool,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    no_compress_index: bool,

    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    no_dirs: bool
}


struct FileInfo
{
    path: PathBuf,
    name: String,
    offest: u64,
    data_size: u64,
    raw_size: u64,
    adler32: u32
}


fn main()
{
    let args = Cli::parse();

    let input = Path::new(&args.input);

    if !input.exists()
    {
        panic!("{}不存在", args.input);
    }

    let mut input_path_list: Vec<PathBuf> = Vec::new();

    if input.is_dir()
    {
        input_path_list = get_all_files_walkdir(input)
    }
    if input.is_file()
    {
        input_path_list.push(input.to_path_buf());
    }

    let mut input_list: Vec<FileInfo> = Vec::new();


    let output: PathBuf;
    if args.output.is_none()
    {
        output = PathBuf::from(format!("{}.xp3",input.file_stem().unwrap().to_string_lossy()));
    }
    else
    {
        output = PathBuf::from(format!("{}.xp3",args.output.unwrap()));
    }

    let mut output_file = File::create(output).expect("打开文件失败");
    output_file.write_all(V230MAGIC.as_slice()).expect("写入失败");

    let index_offset = output_file.stream_position().unwrap();
    println!("{:?}", index_offset);
    output_file.write_all(&[0u8; 8]).expect("写入失败");


    let mut file_info_list: Vec<FileInfo> = Vec::new();
    for i in input_path_list.iter()
    {
        let offset = output_file.stream_position().unwrap();

        let name: String;
        if args.no_dirs
        {
            name = path_str_handle(i.file_name().unwrap().to_string_lossy().to_string())
        }
        else
        {
            name = path_str_handle(i.to_string_lossy().to_string())
        }

        let mut input_file = File::open(i).expect("打开文件失败");
        let alder32: u32;
        if args.no_compress_file
        {
            let mut alder = Adler32::new();
            let mut buf = [0u8; 64 * 1024];
            loop {
                let n = input_file.read(&mut buf).expect("读取文件失败");
                if n == 0 { break; }
                alder.write(&buf[..n]);
                output_file.write_all(&buf[..n]).expect("写入文件失败");
            }
            alder32 = alder.finish() as u32;
        }
        else
        {
            alder32 = compress_stream(input_file, &mut output_file);
        }
        file_info_list.push(FileInfo{
            path: i.to_path_buf(),
            name: name,
            offest: offset,
            data_size: output_file.stream_position().unwrap() - offset,
            raw_size: i.metadata().unwrap().len(),
            adler32: alder32,
        });
    }

    let index_compress_flag = if args.no_compress_index { 0 } else { 1 };
    let file_compress_flag = if args.no_compress_file { 0 } else { 1 };
    let index_header: FileIndexHeader;
    if index_compress_flag == 1
    {
        index_header = FileIndexHeader
        {
            compression_flag: index_compress_flag,
            compression_size: 0,
            raw_size: Some(0),
        };
    }
    else
    {
        index_header = FileIndexHeader
        {
            compression_flag: index_compress_flag,
            compression_size: 0,
            raw_size: None,
        };
    }
    let index_header_offset = output_file.stream_position().unwrap();
    index_header.write(&mut output_file).unwrap();

    let mut index_raw_size: u64 = 0;
    let mut index_compress_size: u64 = 0;
    for i in file_info_list.iter()
    {
        let mut data: Vec<u8> = Vec::new();
        let mut data_cur = Cursor::new(&mut data);
        let info = FileIndexInfo
        {
            entry_size: 4+8+8+2+((i.name.len() as u64)*2),
            flag: 0,
            raw_size: i.raw_size,
            compressed_size: i.data_size,
            name_length: i.name.len() as u16,
            name: i.name.clone()
        };

        let segment_entry = FileIndexSegmentEntry
        {
            flag : file_compress_flag as u32,
            offset: i.offest,
            raw_size: i.raw_size,
            compressed_size: i.data_size,
        };

        let segment = FileIndexSegment
        {
            segment_size : 1*28,
            segment : vec![segment_entry],
        };

        let adler = FileIndexAdler32
        {
            adler32: i.adler32,
        };

        info.write(&mut data_cur).expect("序列化目录数据失败");
        segment.write(&mut data_cur).expect("序列化目录数据失败");
        adler.write(&mut data_cur).expect("序列化目录数据失败");
        let entry_size = data.len();
        let file = FileIndexFile
        {
            entry_size: entry_size as u64,
        };


        if(index_compress_flag == 1)
        {
            let mut file_data: Vec<u8> = Vec::new();
            let mut file_data_cur = Cursor::new(&mut file_data);
            file.write(&mut file_data_cur).expect("序列化目录数据失败");
            file_data.extend(data);
            let index_data = compress(&file_data);
            output_file.write_all(index_data.as_slice()).expect("写入目录数据失败");
            index_raw_size += file_data.len() as u64;
            index_compress_size += index_data.len() as u64;
        }
        else
        {
            let mut header_data: Vec<u8> = Vec::new();
            let mut file_data_cur = Cursor::new(&mut header_data);
            file.write(&mut file_data_cur).expect("写入目录数据失败");
            output_file.write_all(header_data.as_slice()).expect("写入目录数据失败");
            output_file.write_all(data.as_slice()).expect("写入目录数据失败");
            index_raw_size += (header_data.len() + data.len()) as u64;
        }
    }

    let new_index_header: FileIndexHeader;
    if index_compress_flag == 1
    {
        new_index_header = FileIndexHeader
        {
            compression_flag: index_compress_flag,
            compression_size: index_compress_size,
            raw_size: Option::from(index_raw_size),
        };
    }
    else
    {
        new_index_header = FileIndexHeader
        {
            compression_flag: index_compress_flag,
            compression_size: index_raw_size,
            raw_size: None,
        };
    }

    println!("索引头部: {:?}", new_index_header);
    let mut a: Vec<u8> = Vec::new();
    let mut ac = Cursor::new(&mut a);
    output_file.seek(SeekFrom::Start(index_header_offset)).expect("输出文件写入失败");
    new_index_header.write(&mut ac).unwrap();
    println!("{:?}", a);
    output_file.write_all(a.as_slice()).expect("输出文件写入失败");

    output_file.seek(SeekFrom::Start(index_offset)).expect("输出文件写入失败");
    println!("{:?}", index_header_offset.to_le_bytes());
    output_file.write_all(index_header_offset.to_le_bytes().as_slice()).expect("输出文件写入失败");

}