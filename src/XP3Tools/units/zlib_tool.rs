use std::hash::Hasher;
use std::io::{Write, Read, Seek, copy};
use std::process::Output;
use flate2::write::ZlibEncoder;
use flate2::read::ZlibDecoder;
use flate2::Compression;
use adler::Adler32;


pub fn decompress(input: &[u8]) -> Vec<u8> {
    let mut decoder = ZlibDecoder::new(input);
    let mut output = Vec::new();
    decoder.read_to_end(&mut output).expect("解压数据失败");
    output
}


pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(input).expect("压缩数据失败");
    encoder.finish().expect("压缩数据失败")
}


pub fn decompress_stream_vec<R: Read>(input: R) -> Vec<u8>
{
    let mut decode_data = Vec::new();
    let mut decoder = ZlibDecoder::new(input);
    copy(&mut decoder, &mut decode_data).expect("解压数据失败");
    decode_data
}


pub fn decompress_stream<R: Read, W: Write>(input: R, mut output: W)
{
    let mut decoder = ZlibDecoder::new(input);
    copy(&mut decoder, &mut output).expect("解压数据失败");
}


pub fn compress_stream<R: Read, W: Write + Seek>(mut input: R, output: W) -> u32 {
    let mut encoder = ZlibEncoder::new(output, Compression::best());
    let mut buffer = [0u8; 16384];

    let mut adler32 = Adler32::new();

    loop {
        match input.read(&mut buffer) {
            Ok(0) => break, // EOF
            Ok(n) => {
                encoder.write_all(&buffer[..n]).expect("压缩失败");
                adler32.write(&buffer[..n])
            },
            Err(e) => panic!("读取输入时出错: {}", e),
        }
    }

    encoder.finish().expect("压缩失败");
    adler32.finish() as u32
}
