mod units;
use units::path_tool;
use units::constant::*;

use std::path::{Path, PathBuf};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use clap::Parser;

#[derive(Parser)]
#[command(name = "XP3Pack")]
#[command(about = "用于打包xp3", version = "1.0")]
struct Cli {
    input: String,

    #[arg(short, long, default_value = "null")]
    output: String,
}


fn main()
{
    let args = Cli::parse();

    let input = Path::new(&args.input);
    let output = Path::new(&args.output);

    if !input.exists()
    {
        panic!("{}不存在", args.input);
    }

    let mut input_list: Vec<PathBuf> = Vec::new();

    if input.is_dir()
    {
        input_list = path_tool::get_all_files_walkdir(input)
    }
    if input.is_file()
    {
        input_list.push(input.to_path_buf());
    }

    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("output.xp3").expect("输出文件打开失败");

    let mut output_file_writer = BufWriter::new(output_file);

    let len__: i64 = 0;
    output_file_writer.write_all(&MAGIC).expect("写入输出文件失败");
    output_file_writer.write_all(&HEADER).expect("写入输出文件失败");
    output_file_writer.write_all(&0i64.to_le_bytes()).expect("写入输出文件失败");
}