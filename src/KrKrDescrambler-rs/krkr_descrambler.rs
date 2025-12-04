mod units;

use units::header::{Mode, file_mode};
use units::scrambler::{mode0::descramble_mode0, mode1::descramble_mode1, mode2::decompress_zlib};
use units::encode_tools::utf16le_to_utf8;

use clap::Parser;

use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

#[derive(Parser)]
#[command(name = "KrKrDescrambler")]
#[command(about = "用于解混淆ksd文件", version = "1.0")]
struct Cli {
    input: String,

    #[arg(short, long, default_value = "null")]
    output: String,
}

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
        output = format!("{}\\{}.txt",
                             input_path.parent().unwrap().display(),
                             input_path.file_stem().unwrap().to_str().unwrap());
    }
    println!("{}", output);
    let mut output_file = File::create(&output).expect("创建输出文件失败");

    let mut header: [u8; 5] = [0u8; 5];
    input_file.read_exact(&mut header).unwrap();


    let input_mode: Mode = file_mode(header).unwrap();

    match input_mode {
        Mode::MODE0 => {
            let mut data: Vec<u8> = Vec::new();
            input_file.read_to_end(&mut data).expect("读取输入文件失败");
            data = data[5..].to_vec();
            descramble_mode0(&mut data);
            let utf8_data = utf16le_to_utf8(&data);
            output_file.write_all(&utf8_data).expect("写入输出文件失败");
        }
        Mode::MODE1 => {
            let mut data: Vec<u8> = Vec::new();
            input_file.read_to_end(&mut data).expect("读取输入文件失败");
            data = data[5..].to_vec();
            descramble_mode1(&mut data);
            let utf8_data = utf16le_to_utf8(&data);
            output_file.write_all(&utf8_data).expect("写入输出文件失败");
        }
        Mode::MODE2 => {
            let mut data: Vec<u8> = Vec::new();
            input_file.read_to_end(&mut data).expect("读取输入文件失败");
            println!("{:?}", data[..16].to_vec());
            data = data[16..].to_vec();
            let data2 = decompress_zlib(&mut data);
            let utf8_data = utf16le_to_utf8(&data2);
            output_file.write_all(&[0xEF,0xBB,0xBF].as_slice()).unwrap();
            output_file.write_all(&utf8_data).expect("写入输出文件失败");
        }
    }
}