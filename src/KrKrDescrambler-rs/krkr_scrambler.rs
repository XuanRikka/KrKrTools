mod units;

use units::header::{MODE0, MODE1, MODE2};
use units::scrambler::{mode0::scramble_mode0, mode1::scramble_mode1, mode2::compress_zlib};

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use clap::Parser;
use crate::units::encode_tools::utf8_to_utf16le;

#[derive(Parser)]
#[command(name = "KrKrScrambler")]
#[command(about = "用于混淆txt为ksd", version = "1.0")]
struct Cli {
    /// 输入文件路径
    input: String,

    /// 混淆为ksd的模式(0-3)
    #[arg(value_parser = ["0", "1", "2"])]
    mode: String,

    /// 输出文件路径（不填时默认为原文件名+ksd扩展名）
    #[arg(short, long, default_value = "null")]
    output: String,
}

/*
mode2暂时还不可用，其他没测过
*/
fn main()
{
    let args = Cli::parse();

    let input = args.input;
    let mut output = args.output;

    let input_path = Path::new(&input);

    if ! input_path.exists() {
        println!("{} 不存在", &input);
        return;
    }
    let mut input_file = File::open(&input).expect("打开输入文件失败");

    if output == "null"
    {
        output = format!("{}\\{}.ksd",
                         input_path.parent().unwrap().display(),
                         input_path.file_stem().unwrap().to_str().unwrap());
    }
    println!("{}", output);
    let mut output_file = File::create(&output).expect("创建输出文件失败");

    let mut input_data_str = Vec::new();
    input_file.read_to_end(&mut input_data_str).expect("读取输入失败");

    match args.mode.as_str()
    {
        "0" => {
            output_file.write_all(&MODE0).expect("写入文件头失败");
            let mut utf16le_data = utf8_to_utf16le(&input_data_str);
            scramble_mode0(&mut utf16le_data);
            output_file.write_all(&utf16le_data).expect("写入文件失败");
        },
        "1" => {
            output_file.write_all(&MODE1).expect("写入文件头失败");
            let mut utf16le_data = utf8_to_utf16le(&input_data_str);
            scramble_mode1(&mut utf16le_data);
            output_file.write_all(&utf16le_data).expect("写入文件失败");
        },
        "2" => {
            output_file.write_all(&MODE2).expect("写入文件头失败");
            let mut utf16le_data = utf8_to_utf16le(&input_data_str);
            let compress_data = compress_zlib(&mut utf16le_data);
            output_file.write_all(&compress_data).expect("写入文件失败");
        },
        _ => {
            return;
        }
    }
}