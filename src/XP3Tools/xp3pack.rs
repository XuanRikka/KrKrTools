mod units;
use units::path_tool::*;
use units::models::*;
use units::zlib_tool::{compress_stream, compress, compress_stream_zopfli, compress_zopfli};

use std::path::{PathBuf};
use std::fs::{File};
use std::hash::Hasher;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};
use std::env;

use adler::Adler32;
use binrw::BinWrite;
use clap::Parser;
use clap;

/// 用于打包XP3文件的工具
///
/// 默认行为为压缩文件内容及其目录，并且保持目录结构
#[derive(Parser)]
#[command(version,about,long_about = None)]
struct Cli {
    /// 输入文件或者目录
    input: Vec<PathBuf>,

    /// 输出的xp3文件名称（可选）
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 打包时的基准路径，即生成的xp3文件内的索引数据内的路径的基准路径，默认为程序执行路径
    #[arg(short, long)]
    base: Option<PathBuf>,

    /// 使用zopfli算法来压缩，能提高压缩率，但是会极大增加压缩时间
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    zopfli: bool,

    /// 添加该选项时不压缩被打包的文件内容
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    no_compress_file: bool,

    /// 添加该选项时不压缩目录索引
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    no_compress_index: bool,

    /// 添加该选项时不保持目录结构
    #[arg(long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    no_dirs: bool
}


#[derive(Debug)]
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

    let cwd = get_cwd();
    let base;
    if args.base.is_none()
    {
        base = get_cwd();
    }
    else
    {
        base = args.base.unwrap();
        env::set_current_dir(&base).expect("设置工作目录失败");
    }

    let mut inputs: Vec<PathBuf> = Vec::new();
    for path_str in args.input {
        if !path_str.exists() {
            panic!("{} 不存在", path_str.display());
        }
        inputs.push(absolute_to_relative(&base, &path_str));
    }

    let mut input_path_list: Vec<PathBuf> = Vec::new();

    for input in inputs.iter()
    {
        if input.is_dir()
        {
            input_path_list.extend(get_all_files_walkdir(input))
        }
        if input.is_file()
        {
            input_path_list.push(input.to_path_buf());
        }
    }


    let mut output: PathBuf;
    if args.output.is_none()
    {
        output = cwd.join(PathBuf::from("output.xp3"));
    }
    else
    {
        output = args.output.unwrap();
        if output.is_relative()
        {
            output = cwd.join(output);
        }
    }
    println!("输出文件：{}", output.display());

    let mut output_file = File::create(output).expect("打开文件失败");
    output_file.write_all(V230MAGIC.as_slice()).expect("写入失败");

    let index_offset = output_file.stream_position().unwrap();

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
        else if !args.no_compress_file&&!args.zopfli
        {
            alder32 = compress_stream(input_file, &mut output_file);
        }
        else
        {
            alder32 = compress_stream_zopfli(&mut input_file, &mut output_file);
        }
        file_info_list.push(FileInfo{
            path: i.to_path_buf(),
            name: name.clone(),
            offest: offset,
            data_size: output_file.stream_position().unwrap() - offset,
            raw_size: i.metadata().unwrap().len(),
            adler32: alder32,
        });
        println!("打包 {} -> {}", i.to_string_lossy(), name)
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
    let mut index_data: Vec<u8> = Vec::new();
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


        let mut header_data: Vec<u8> = Vec::new();
        let mut file_data_cur = Cursor::new(&mut header_data);
        file.write(&mut file_data_cur).expect("写入目录数据失败");
        index_data.write_all(header_data.as_slice()).expect("写入目录数据失败");
        index_data.write_all(data.as_slice()).expect("写入目录数据失败");
    }
    index_raw_size =  index_data.len() as u64;
    if index_compress_flag == 1&&!args.zopfli
    {
        let compress_data = compress(index_data.as_slice());
        index_compress_size = compress_data.len() as u64;
        output_file.write_all(compress_data.as_slice()).expect("写入目录数据失败");
    }
    else if index_compress_flag == 1&&args.zopfli
    {
        let compress_data = compress_zopfli(index_data.as_slice());
        index_compress_size = compress_data.len() as u64;
        output_file.write_all(compress_data.as_slice()).expect("写入目录数据失败");
    }
    else
    {
        output_file.write_all(index_data.as_slice()).expect("写入目录数据失败");
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

    let mut a: Vec<u8> = Vec::new();
    let mut ac = Cursor::new(&mut a);
    output_file.seek(SeekFrom::Start(index_header_offset)).expect("输出文件写入失败");
    new_index_header.write(&mut ac).unwrap();
    output_file.write_all(a.as_slice()).expect("输出文件写入失败");

    output_file.seek(SeekFrom::Start(index_offset)).expect("输出文件写入失败");
    output_file.write_all(index_header_offset.to_le_bytes().as_slice()).expect("输出文件写入失败");

}