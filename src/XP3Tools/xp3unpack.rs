mod units;

use std::fs::{File, create_dir_all};
use std::io::Write;
use units::xp3parse::XP3Parser;

use std::path::Path;

use clap::Parser;


#[derive(Parser)]
#[command(name = "xp3unpack", version = "1.0", about = "示例程序")]
struct Cli
{
    input: String,

    #[arg(short, long)]
    output: Option<String>,
}


fn main()
{
    let args = Cli::parse();
    let input = Path::new(&args.input);

    if !input.exists()
    {
        panic!("输入的路径不存在")
    }
    if !input.is_file()
    {
        panic!("输入的路径不是文件")
    }

    let out = args.output.unwrap_or(
        input.file_stem().unwrap().to_string_lossy().parse().unwrap()
    );

    let output = Path::new(&out);
    create_dir_all(output).unwrap();

    let file = File::open(input).unwrap();

    let mut xp3 = XP3Parser::new(file);

    let file_name_vec = xp3.get_file_name();

    for name in file_name_vec
    {
        let output_file = output.join(Path::new(&name));
        create_dir_all(output_file.parent().unwrap()).unwrap();
        let file = File::create(output_file.clone()).unwrap();
        xp3.get_file(&*name, file);
        println!("解包 {} -> {}", name, output_file.display())
    }

}